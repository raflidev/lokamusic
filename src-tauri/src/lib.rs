mod scanner;
mod types;

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use std::time::{Duration, Instant, SystemTime};
use discord_rich_presence::{activity, activity::ActivityType, DiscordIpc, DiscordIpcClient};
use urlencoding::encode as url_encode;
use serde_json::json;
use tauri::{
    image::Image,
    menu::{IconMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::{TrayIcon, TrayIconBuilder},
    Emitter, Manager,
};
use tauri_plugin_store::StoreExt;
use types::*;

const DISCORD_CLIENT_ID: &str = "1508866364251574342";

struct DiscordState {
    client: Option<DiscordIpcClient>,
    artwork_cache: HashMap<String, String>,
}

impl DiscordState {
    fn new() -> Self {
        DiscordState { client: None, artwork_cache: HashMap::new() }
    }

    fn ensure_connected(&mut self) -> bool {
        if self.client.is_some() {
            return true;
        }
        match DiscordIpcClient::new(DISCORD_CLIENT_ID) {
            Ok(mut c) => {
                if c.connect().is_ok() {
                    self.client = Some(c);
                    true
                } else {
                    false
                }
            }
            Err(_) => false,
        }
    }

    fn set_activity(&mut self, details: &str, state_str: &str, start_ts: Option<i64>, artwork_url: &str) {
        if !self.ensure_connected() {
            return;
        }
        if let Some(ref mut client) = self.client {
            let assets = activity::Assets::new()
                .large_image(artwork_url)
                .small_image("lokamusic_logo");

            let mut act = activity::Activity::new()
                .activity_type(ActivityType::Listening)
                .details(details)
                .state(state_str)
                .assets(assets);

            if let Some(ts) = start_ts {
                act = act.timestamps(activity::Timestamps::new().start(ts));
            }

            if client.set_activity(act).is_err() {
                self.client = None;
            }
        }
    }

    fn clear(&mut self) {
        if let Some(ref mut client) = self.client {
            let _ = client.clear_activity();
        }
    }
}

static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

fn http_client() -> &'static reqwest::Client {
    HTTP_CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent(concat!("lokamusic/", env!("CARGO_PKG_VERSION")))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new())
    })
}

async fn fetch_artwork_url(artist: &str, album: &str) -> String {
    let query = format!("{} {}", artist, album);
    let encoded = url_encode(&query);
    let url = format!(
        "https://itunes.apple.com/search?term={}&entity=album&limit=1",
        encoded
    );

    let Ok(res) = http_client().get(&url).send().await else { return "lokamusic_logo".into() };
    let Ok(json) = res.json::<serde_json::Value>().await else { return "lokamusic_logo".into() };

    json["results"][0]["artworkUrl100"]
        .as_str()
        .map(|u| u.replace("100x100bb", "512x512bb"))
        .unwrap_or_else(|| "lokamusic_logo".into())
}

const STORE_FILE: &str = "config.json";

// Serializes read-modify-write store operations. Without this, concurrent
// commands (e.g. a scan finishing while the user likes a song) can overwrite
// each other's changes.
static STORE_LOCK: Mutex<()> = Mutex::new(());

fn store_guard() -> MutexGuard<'static, ()> {
    STORE_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

fn save_store_now(app: &tauri::AppHandle) {
    if let Ok(store) = app.store(STORE_FILE) {
        if let Err(e) = store.save() {
            log::error!("failed to save store: {e}");
        }
    }
}

// --- Store helpers ---

fn store_get<T: serde::de::DeserializeOwned>(app: &tauri::AppHandle, key: &str) -> Vec<T> {
    let store = match app.store(STORE_FILE) {
        Ok(store) => store,
        Err(e) => {
            log::error!("failed to open store: {e}");
            return Vec::new();
        }
    };
    match store.get(key) {
        Some(value) => serde_json::from_value(value).unwrap_or_else(|e| {
            log::error!("failed to deserialize store key '{key}': {e}");
            Vec::new()
        }),
        None => Vec::new(),
    }
}

fn store_get_obj<T: serde::de::DeserializeOwned + Default>(app: &tauri::AppHandle, key: &str) -> T {
    let store = match app.store(STORE_FILE) {
        Ok(store) => store,
        Err(e) => {
            log::error!("failed to open store: {e}");
            return T::default();
        }
    };
    match store.get(key) {
        Some(value) => serde_json::from_value(value).unwrap_or_else(|e| {
            log::error!("failed to deserialize store key '{key}': {e}");
            T::default()
        }),
        None => T::default(),
    }
}

fn store_set<T: serde::Serialize>(app: &tauri::AppHandle, key: &str, value: &T) {
    let store = match app.store(STORE_FILE) {
        Ok(store) => store,
        Err(e) => {
            log::error!("failed to open store: {e}");
            return;
        }
    };
    match serde_json::to_value(value) {
        // tauri-plugin-store debounces disk writes (~100ms) on every set, so
        // there is no need to call save() here for each mutation.
        Ok(value) => store.set(key, value),
        Err(e) => log::error!("failed to serialize store key '{key}': {e}"),
    }
}

/// Grants the asset protocol access to the media the app has stored. The
/// protocol scope starts empty (see `tauri.conf.json`) so a compromised
/// renderer cannot read arbitrary files.
fn allow_asset_directory(app: &tauri::AppHandle, path: impl AsRef<std::path::Path>, recursive: bool) {
    if let Err(e) = app.asset_protocol_scope().allow_directory(path, recursive) {
        log::warn!("failed to allow asset directory: {e}");
    }
}

fn allow_stored_media(app: &tauri::AppHandle) {
    let folders: Vec<WatchedFolder> = store_get(app, "folders");
    for folder in &folders {
        allow_asset_directory(app, &folder.path, true);
    }

    // Directly imported songs only need their own parent directory.
    let songs: Vec<Song> = store_get(app, "songs");
    let mut imported_dirs: HashSet<String> = HashSet::new();
    for song in songs.iter().filter(|song| song.folder_id.is_none()) {
        if let Some(parent) = PathBuf::from(&song.path).parent() {
            let parent = parent.to_string_lossy().to_string();
            if imported_dirs.insert(parent.clone()) {
                allow_asset_directory(app, &parent, false);
            }
        }
    }
}

// --- Path helpers ---

/// Normalizes separators (and case on Windows) before comparing paths.
fn normalize_path(path: &str) -> String {
    let normalized = path.replace('\\', "/");
    if cfg!(windows) {
        normalized.to_lowercase()
    } else {
        normalized
    }
}

/// Returns `true` when `child` is `parent` itself or lives inside it.
/// Compares path components so `/music/rock` does not match `/music/rockabilly`.
fn path_is_within(child: &str, parent: &str) -> bool {
    let child = normalize_path(child);
    let parent = normalize_path(parent);
    let parent = parent.trim_end_matches('/');
    if parent.is_empty() {
        return true;
    }
    child == parent
        || child
            .strip_prefix(parent)
            .is_some_and(|rest| rest.starts_with('/'))
}

fn paths_equal(a: &str, b: &str) -> bool {
    normalize_path(a.trim_end_matches('/')) == normalize_path(b.trim_end_matches('/'))
}

fn now_ms() -> f64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as f64
}

// --- Commands ---

#[tauri::command]
async fn dialog_select_folder(app: tauri::AppHandle) -> Result<Option<String>, String> {
    tokio::task::spawn_blocking(move || {
        use tauri_plugin_dialog::DialogExt;
        Ok(app
            .dialog()
            .file()
            .blocking_pick_folder()
            .and_then(|p| p.into_path().ok())
            .map(|p| p.to_string_lossy().to_string()))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn dialog_select_files(app: tauri::AppHandle) -> Result<Vec<String>, String> {
    tokio::task::spawn_blocking(move || {
        use tauri_plugin_dialog::DialogExt;
        let files = app
            .dialog()
            .file()
            .add_filter("Audio", &["mp3", "flac", "wav", "ogg", "m4a", "aac", "wma", "opus"])
            .blocking_pick_files();
        let paths = files
            .unwrap_or_default()
            .into_iter()
            .filter_map(|p| p.into_path().ok())
            .map(|p| p.to_string_lossy().to_string())
            .collect();
        Ok(paths)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn library_get_song_art(path: String) -> Option<String> {
    tokio::task::spawn_blocking(move || scanner::read_song_art(&PathBuf::from(&path)))
        .await
        .ok()
        .flatten()
}

#[tauri::command]
fn library_get_all_songs(app: tauri::AppHandle) -> Vec<Song> {
    store_get(&app, "songs")
}

#[tauri::command]
fn library_get_folders(app: tauri::AppHandle) -> Vec<WatchedFolder> {
    store_get(&app, "folders")
}

#[tauri::command]
async fn library_add_folder(path: String, app: tauri::AppHandle) -> Result<Option<WatchedFolder>, String> {
    let path = path.replace('\\', "/");

    let folder = {
        let _guard = store_guard();
        let mut folders: Vec<WatchedFolder> = store_get(&app, "folders");
        if folders.iter().any(|f| paths_equal(&f.path, &path)) {
            return Ok(None);
        }

        let id = format!("{:x}", md5::compute(&path));
        let name = PathBuf::from(&path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&path)
            .to_string();

        let folder = WatchedFolder {
            id,
            path: path.clone(),
            name,
            song_count: 0,
            size_bytes: 0,
            last_scanned: None,
        };

        folders.push(folder.clone());
        store_set(&app, "folders", &folders);
        folder
    };

    allow_asset_directory(&app, &path, true);

    let app_clone = app.clone();
    let folder_id = folder.id.clone();
    tauri::async_runtime::spawn(async move {
        run_scan(&app_clone, folder_id, path).await;
    });

    Ok(Some(folder))
}

#[tauri::command]
async fn library_remove_folder(id: String, app: tauri::AppHandle) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = store_guard();
        let folders: Vec<WatchedFolder> = store_get(&app, "folders");
        let folder = folders.iter().find(|f| f.id == id).cloned();
        let updated_folders: Vec<WatchedFolder> = folders.into_iter().filter(|f| f.id != id).collect();
        store_set(&app, "folders", &updated_folders);

        if let Some(f) = folder {
            // Songs that belong to this folder are removed. Legacy entries without
            // a folder id fall back to a component-aware path check.
            let songs: Vec<Song> = store_get::<Song>(&app, "songs")
                .into_iter()
                .filter(|s| match s.folder_id.as_deref() {
                    Some(folder_id) => folder_id != id,
                    None => !path_is_within(&s.path, &f.path),
                })
                .collect();
            store_set(&app, "songs", &songs);
            app.emit("library:songs-updated", &songs).ok();
        }

        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn library_scan_folder(id: String, app: tauri::AppHandle) -> Result<(), String> {
    let folders: Vec<WatchedFolder> = store_get(&app, "folders");
    if let Some(folder) = folders.iter().find(|f| f.id == id) {
        let folder_path = folder.path.clone();
        let app_clone = app.clone();
        tauri::async_runtime::spawn(async move {
            run_scan(&app_clone, id, folder_path).await;
        });
    }
    Ok(())
}

#[tauri::command]
async fn library_import_files(paths: Vec<String>, app: tauri::AppHandle) -> Result<ImportResult, String> {
    let mut imported: Vec<Song> = Vec::with_capacity(paths.len());
    let mut errors = 0u32;

    for path_str in &paths {
        let path = PathBuf::from(path_str);
        match tokio::task::spawn_blocking(move || scanner::read_file_meta(&path)).await {
            Ok(song) => imported.push(song),
            Err(_) => errors += 1,
        }
    }

    let added = tauri::async_runtime::spawn_blocking(move || {
        let _guard = store_guard();
        let mut songs: Vec<Song> = store_get(&app, "songs");
        // Keep this set updated so duplicate paths within the same import are skipped.
        let mut existing_ids: HashSet<String> = songs.iter().map(|s| s.id.clone()).collect();
        let folders: Vec<WatchedFolder> = store_get(&app, "folders");
        let mut added = 0u32;
        let mut allowed_dirs: HashSet<String> = HashSet::new();

        for mut song in imported {
            if !existing_ids.insert(song.id.clone()) {
                continue;
            }
            // Imported files that live inside a watched folder still belong to it.
            song.folder_id = folders
                .iter()
                .filter(|f| path_is_within(&song.path, &f.path))
                .max_by_key(|f| f.path.len())
                .map(|f| f.id.clone());
            if let Some(parent) = PathBuf::from(&song.path).parent() {
                let parent = parent.to_string_lossy().to_string();
                if allowed_dirs.insert(parent.clone()) {
                    allow_asset_directory(&app, &parent, false);
                }
            }
            songs.push(song);
            added += 1;
        }

        store_set(&app, "songs", &songs);
        app.emit("library:songs-updated", &songs).ok();

        let mut history: Vec<ScanEvent> = store_get(&app, "scanHistory");
        history.insert(0, ScanEvent {
            timestamp: now_ms(),
            folder_path: "Individual files".to_string(),
            success: added,
            errors,
        });
        history.truncate(50);
        store_set(&app, "scanHistory", &history);

        added
    })
    .await
    .map_err(|e| e.to_string())?;

    Ok(ImportResult { added, errors })
}

#[tauri::command]
async fn library_toggle_like(song_id: String, app: tauri::AppHandle) -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = store_guard();
        let mut songs: Vec<Song> = store_get(&app, "songs");
        let mut liked = false;
        for s in &mut songs {
            if s.id == song_id {
                s.liked = !s.liked;
                liked = s.liked;
                break;
            }
        }
        store_set(&app, "songs", &songs);
        liked
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
async fn library_set_lyrics(song_id: String, lyrics: String, app: tauri::AppHandle) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = store_guard();
        let mut songs: Vec<Song> = store_get(&app, "songs");
        for s in &mut songs {
            if s.id == song_id {
                s.lyrics = Some(lyrics);
                break;
            }
        }
        store_set(&app, "songs", &songs);
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
async fn library_fetch_lyrics(
    song_id: String,
    title: String,
    artist: String,
    album: String,
    duration: f64,
    app: tauri::AppHandle,
) -> Result<String, String> {
    let url = format!(
        "https://lrclib.net/api/get?track_name={}&artist_name={}&album_name={}&duration={}",
        url_encode(&title),
        url_encode(&artist),
        url_encode(&album),
        duration.round() as i64,
    );

    let res = http_client().get(&url).send().await.map_err(|e| e.to_string())?;
    if res.status().as_u16() == 404 {
        return Err("Lyrics not found".to_string());
    }
    if !res.status().is_success() {
        return Err(format!("API error: {}", res.status()));
    }

    let data: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
    let lyrics = data["syncedLyrics"]
        .as_str()
        .or_else(|| data["plainLyrics"].as_str())
        .ok_or_else(|| "No lyrics available".to_string())?
        .to_string();

    {
        let _guard = store_guard();
        let mut songs: Vec<Song> = store_get(&app, "songs");
        for s in &mut songs {
            if s.id == song_id {
                s.lyrics = Some(lyrics.clone());
                break;
            }
        }
        store_set(&app, "songs", &songs);
    }

    Ok(lyrics)
}

#[tauri::command]
async fn library_update_play(song_id: String, app: tauri::AppHandle) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let _guard = store_guard();
        let mut songs: Vec<Song> = store_get(&app, "songs");
        for s in &mut songs {
            if s.id == song_id {
                s.play_count += 1;
                s.last_played = Some(now_ms());
                break;
            }
        }
        store_set(&app, "songs", &songs);
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn playlist_get_all(app: tauri::AppHandle) -> Vec<Playlist> {
    store_get(&app, "playlists")
}

#[tauri::command]
async fn playlist_create(name: String, app: tauri::AppHandle) -> Result<Playlist, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = store_guard();
        let mut playlists: Vec<Playlist> = store_get(&app, "playlists");
        let playlist = Playlist {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            song_ids: vec![],
            created_at: now_ms(),
        };
        playlists.push(playlist.clone());
        store_set(&app, "playlists", &playlists);
        playlist
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
async fn playlist_delete(id: String, app: tauri::AppHandle) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = store_guard();
        let playlists: Vec<Playlist> = store_get::<Playlist>(&app, "playlists")
            .into_iter()
            .filter(|p| p.id != id)
            .collect();
        store_set(&app, "playlists", &playlists);
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
async fn playlist_rename(id: String, name: String, app: tauri::AppHandle) -> Result<Option<Playlist>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = store_guard();
        let mut playlists: Vec<Playlist> = store_get(&app, "playlists");
        let mut found: Option<Playlist> = None;
        for p in &mut playlists {
            if p.id == id {
                p.name = name.clone();
                found = Some(p.clone());
                break;
            }
        }
        store_set(&app, "playlists", &playlists);
        found
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
async fn playlist_add_song(playlist_id: String, song_id: String, app: tauri::AppHandle) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = store_guard();
        let mut playlists: Vec<Playlist> = store_get(&app, "playlists");
        for p in &mut playlists {
            if p.id == playlist_id && !p.song_ids.contains(&song_id) {
                p.song_ids.push(song_id);
                break;
            }
        }
        store_set(&app, "playlists", &playlists);
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
async fn playlist_remove_song(playlist_id: String, song_id: String, app: tauri::AppHandle) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = store_guard();
        let mut playlists: Vec<Playlist> = store_get(&app, "playlists");
        for p in &mut playlists {
            if p.id == playlist_id {
                p.song_ids.retain(|id| id != &song_id);
                break;
            }
        }
        store_set(&app, "playlists", &playlists);
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn open_miniplayer(app: tauri::AppHandle) -> Result<(), String> {
    use tauri::{WebviewWindowBuilder, WebviewUrl};

    if let Some(w) = app.get_webview_window("miniplayer") {
        let _ = w.show();
        let _ = w.set_focus();
        return Ok(());
    }

    let builder = WebviewWindowBuilder::new(&app, "miniplayer", WebviewUrl::App("index.html".into()))
        .title("lokamusic")
        .inner_size(320.0, 380.0)
        .resizable(true)
        .min_inner_size(240.0, 330.0)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true);

    #[cfg(target_os = "macos")]
    let builder = builder.visible_on_all_workspaces(true);

    builder.build().map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
fn close_miniplayer(app: tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("miniplayer") {
        let _ = w.close();
    }
}

fn open_tray_popover(app: &tauri::AppHandle, rect: tauri::Rect) {
    use tauri::{LogicalPosition, WebviewWindowBuilder, WebviewUrl};

    const WIDTH: f64 = 300.0;
    const HEIGHT: f64 = 190.0;
    const MARGIN: f64 = 6.0;

    // The tray rect is physical; locate the monitor under the icon so the
    // popover is positioned correctly on multi-monitor / mixed-DPI setups.
    let primary_scale = app
        .primary_monitor()
        .ok()
        .flatten()
        .map(|m| m.scale_factor())
        .unwrap_or(1.0);
    let icon_physical = rect.position.to_physical::<f64>(primary_scale);
    let monitor = app
        .get_webview_window("main")
        .and_then(|w| w.monitor_from_point(icon_physical.x, icon_physical.y).ok().flatten())
        .or_else(|| app.primary_monitor().ok().flatten());

    let scale = monitor.as_ref().map(|m| m.scale_factor()).unwrap_or(primary_scale);
    let monitor_pos = monitor
        .as_ref()
        .map(|m| m.position().to_logical::<f64>(scale))
        .unwrap_or_else(|| LogicalPosition::new(0.0, 0.0));
    let monitor_size = monitor
        .as_ref()
        .map(|m| m.size().to_logical::<f64>(scale))
        .unwrap_or_else(|| tauri::LogicalSize::new(WIDTH, HEIGHT));

    let icon_pos = rect.position.to_logical::<f64>(scale);
    let icon_size = rect.size.to_logical::<f64>(scale);

    let mut x = icon_pos.x + icon_size.width / 2.0 - WIDTH / 2.0;
    let mut y = icon_pos.y + icon_size.height + 4.0;

    // If the tray sits at the bottom of the screen, open the popover above it.
    if y + HEIGHT > monitor_pos.y + monitor_size.height - MARGIN {
        y = icon_pos.y - HEIGHT - 4.0;
    }

    let min_x = monitor_pos.x + MARGIN;
    let max_x = monitor_pos.x + monitor_size.width - WIDTH - MARGIN;
    if max_x >= min_x {
        x = x.clamp(min_x, max_x);
    }

    let min_y = monitor_pos.y + MARGIN;
    let max_y = monitor_pos.y + monitor_size.height - HEIGHT - MARGIN;
    if max_y >= min_y {
        y = y.clamp(min_y, max_y);
    }

    let builder = WebviewWindowBuilder::new(app, "tray-popover", WebviewUrl::App("index.html".into()))
        .title("lokamusic")
        .inner_size(WIDTH, HEIGHT)
        .resizable(false)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .position(x, y);

    #[cfg(target_os = "macos")]
    let builder = builder.visible_on_all_workspaces(true);

    let _ = builder.build();
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() > max {
        format!("{}…", s.chars().take(max - 1).collect::<String>())
    } else {
        s.to_string()
    }
}

fn decode_art(data_uri: &str) -> Option<Image<'static>> {
    use base64::{engine::general_purpose::STANDARD, Engine};
    let b64 = data_uri.split(',').nth(1)?;
    let bytes = STANDARD.decode(b64).ok()?;
    Image::from_bytes(&bytes).ok().map(|i| i.to_owned())
}

#[tauri::command]
fn tray_set_now_playing(
    payload: Option<serde_json::Value>,
    tray: tauri::State<TrayIcon>,
    now_playing_item: tauri::State<IconMenuItem<tauri::Wry>>,
) -> Result<(), String> {
    let Some(p) = payload.filter(|p| !p["title"].as_str().unwrap_or("").is_empty()) else {
        tray.set_title(None::<&str>).map_err(|e| e.to_string())?;
        now_playing_item.set_text("Not Playing").map_err(|e| e.to_string())?;
        now_playing_item.set_icon(None).map_err(|e| e.to_string())?;
        return Ok(());
    };

    let title = p["title"].as_str().unwrap_or("Unknown").to_string();
    let artist = p["artist"].as_str().unwrap_or("Unknown Artist").to_string();
    let album = p["album"].as_str().unwrap_or("").to_string();
    let playing = p["isPlaying"].as_bool().unwrap_or(false);
    let art = p["art"].as_str().and_then(decode_art);

    let status_line = format!("{} {} - {}", if playing { "♪" } else { "⏸" }, title, artist);
    tray.set_title(Some(truncate(&status_line, 40))).map_err(|e| e.to_string())?;

    let menu_text = if album.is_empty() {
        format!("{}\n{}", title, artist)
    } else {
        format!("{}\n{} - {}", title, artist, album)
    };
    now_playing_item.set_text(menu_text).map_err(|e| e.to_string())?;
    now_playing_item.set_icon(art).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
fn settings_get(app: tauri::AppHandle) -> Settings {
    store_get_obj(&app, "settings")
}

#[tauri::command]
async fn settings_set_playback(
    volume: f64,
    shuffle: bool,
    repeat: String,
    app: tauri::AppHandle,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = store_guard();
        let mut settings: Settings = store_get_obj(&app, "settings");
        settings.volume = volume.clamp(0.0, 1.0);
        settings.shuffle = shuffle;
        settings.repeat = match repeat.as_str() {
            "one" | "all" => repeat,
            _ => "none".to_string(),
        };
        store_set(&app, "settings", &settings);
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
async fn settings_set_analytics(enabled: bool, app: tauri::AppHandle) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = store_guard();
        let mut settings: Settings = store_get_obj(&app, "settings");
        settings.analytics = enabled;
        store_set(&app, "settings", &settings);
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
async fn settings_set_discord_presence(
    enabled: bool,
    app: tauri::AppHandle,
    discord: tauri::State<'_, Arc<Mutex<DiscordState>>>,
) -> Result<(), String> {
    {
        let _guard = store_guard();
        let mut settings: Settings = store_get_obj(&app, "settings");
        settings.discord_presence = enabled;
        store_set(&app, "settings", &settings);
    }

    if !enabled {
        let arc = Arc::clone(&discord);
        tokio::task::spawn_blocking(move || {
            if let Ok(mut d) = arc.lock() {
                d.clear();
            }
        })
        .await
        .ok();
    }

    Ok(())
}

enum DiscordAction {
    SetActivity { title: String, state_str: String, start_ts: i64, artwork_url: String },
    Clear,
}

#[tauri::command]
async fn discord_update_presence(
    payload: Option<serde_json::Value>,
    app: tauri::AppHandle,
    discord: tauri::State<'_, Arc<Mutex<DiscordState>>>,
) -> Result<(), String> {
    let settings: Settings = store_get_obj(&app, "settings");
    if !settings.discord_presence {
        return Ok(());
    }

    let action = match payload {
        Some(ref p) if p["isPlaying"].as_bool().unwrap_or(false) => {
            let title = p["title"].as_str().unwrap_or("Unknown").to_string();
            let artist = p["artist"].as_str().unwrap_or("Unknown Artist").to_string();
            let album = p["album"].as_str().unwrap_or("Unknown Album").to_string();
            let current_time = p["currentTime"].as_f64().unwrap_or(0.0);
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;

            let cache_key = format!("{}::{}", artist, album);
            let artwork_url = {
                let cached = discord.lock().ok()
                    .and_then(|d| d.artwork_cache.get(&cache_key).cloned());
                match cached {
                    Some(url) => url,
                    None => {
                        let url = fetch_artwork_url(&artist, &album).await;
                        if let Ok(mut d) = discord.lock() {
                            // Keep the artwork cache bounded during long sessions.
                            if d.artwork_cache.len() >= 256 {
                                d.artwork_cache.clear();
                            }
                            d.artwork_cache.insert(cache_key, url.clone());
                        }
                        url
                    }
                }
            };

            DiscordAction::SetActivity {
                state_str: format!("{} - {}", artist, album),
                start_ts: now - current_time as i64,
                title,
                artwork_url,
            }
        }
        _ => DiscordAction::Clear,
    };

    let arc = Arc::clone(&discord);
    tokio::task::spawn_blocking(move || {
        if let Ok(mut d) = arc.lock() {
            match action {
                DiscordAction::SetActivity { title, state_str, start_ts, artwork_url } => {
                    d.set_activity(&title, &state_str, Some(start_ts), &artwork_url);
                }
                DiscordAction::Clear => d.clear(),
            }
        }
    })
    .await
    .ok();

    Ok(())
}

// --- Background scan ---

async fn run_scan(app: &tauri::AppHandle, folder_id: String, folder_path: String) {
    let path = PathBuf::from(&folder_path);

    let (tx, mut rx) = tokio::sync::mpsc::channel::<(u32, u32, u32)>(64);
    let folder_id_for_scan = folder_id.clone();

    let scan_handle = tokio::task::spawn_blocking(move || {
        let (files, size_bytes) = scanner::collect_audio_files(&path);
        let total = files.len() as u32;
        let mut songs: Vec<Song> = Vec::with_capacity(files.len());
        let mut last_report = Instant::now();

        for (i, file) in files.iter().enumerate() {
            songs.push(scanner::read_file_meta(file));
            let scanned = (i + 1) as u32;
            // Coalesce progress updates instead of emitting one event per file.
            if scanned == total || scanned % 25 == 0 || last_report.elapsed() >= Duration::from_millis(100) {
                let percent = scanned * 100 / total.max(1);
                let _ = tx.blocking_send((scanned, total, percent));
                last_report = Instant::now();
            }
        }

        (songs, size_bytes)
    });

    while let Some((scanned, total, percent)) = rx.recv().await {
        app.emit("library:scan-progress", ScanProgressPayload {
            folder_id: folder_id.clone(),
            scanned,
            total,
            percent,
        }).ok();
    }

    let (new_songs, size_bytes) = match scan_handle.await {
        Ok(r) => r,
        Err(_) => {
            app.emit("library:scan-complete", ScanCompletePayload {
                folder_id,
                songs: store_get(app, "songs"),
                folders: store_get(app, "folders"),
            }).ok();
            return;
        }
    };

    // Merge: preserve liked/playCount/lastPlayed from existing songs. If the
    // folder was removed while scanning, discard the result instead of
    // resurrecting its songs.
    let errors = 0u32;
    let merge_result = {
        let _guard = store_guard();
        let mut folders: Vec<WatchedFolder> = store_get(app, "folders");

        if !folders.iter().any(|f| f.id == folder_id_for_scan) {
            None
        } else {
            let existing: Vec<Song> = store_get(app, "songs");
            let existing_map: HashMap<String, Song> = existing.into_iter().map(|s| (s.id.clone(), s)).collect();

            let kept: Vec<Song> = existing_map.values()
                .filter(|s| match s.folder_id.as_deref() {
                    Some(folder_id) => folder_id != folder_id_for_scan,
                    None => !path_is_within(&s.path, &folder_path),
                })
                .cloned()
                .collect();

            let updated: Vec<Song> = new_songs.into_iter().map(|mut s| {
                s.folder_id = Some(folder_id_for_scan.clone());
                if let Some(prev) = existing_map.get(&s.id) {
                    s.liked = prev.liked;
                    s.play_count = prev.play_count;
                    s.last_played = prev.last_played;
                }
                s
            }).collect();

            let mut merged = kept;
            merged.extend(updated);
            store_set(app, "songs", &merged);

            let song_count = merged.iter()
                .filter(|s| s.folder_id.as_deref() == Some(folder_id_for_scan.as_str()))
                .count() as u32;
            for f in &mut folders {
                if f.id == folder_id_for_scan {
                    f.song_count = song_count;
                    f.size_bytes = size_bytes;
                    f.last_scanned = Some(now_ms());
                    break;
                }
            }
            store_set(app, "folders", &folders);

            let mut history: Vec<ScanEvent> = store_get(app, "scanHistory");
            history.insert(0, ScanEvent {
                timestamp: now_ms(),
                folder_path: folder_path.clone(),
                success: song_count,
                errors,
            });
            history.truncate(50);
            store_set(app, "scanHistory", &history);

            Some((merged, folders))
        }
    };

    let (songs, folders) = match merge_result {
        Some(result) => result,
        None => (store_get(app, "songs"), store_get(app, "folders")),
    };

    app.emit("library:scan-complete", ScanCompletePayload {
        folder_id: folder_id_for_scan,
        songs,
        folders,
    }).ok();
}

// --- App setup ---

/// One-time normalization performed on startup:
/// - assigns `folder_id` to songs stored by older versions,
/// - drops songs belonging to folders that no longer exist (directly imported
///   songs have no folder and are kept),
/// - rewrites the store without the legacy base64 `albumArt` payload.
fn migrate_library(app: &tauri::AppHandle) {
    const SCHEMA_VERSION: u64 = 2;

    let _guard = store_guard();
    let store = match app.store(STORE_FILE) {
        Ok(store) => store,
        Err(e) => {
            log::error!("failed to open store: {e}");
            return;
        }
    };
    let version = store.get("schemaVersion").and_then(|v| v.as_u64()).unwrap_or(0);
    if version >= SCHEMA_VERSION {
        return;
    }

    let folders: Vec<WatchedFolder> = match store.get("folders") {
        Some(value) => match serde_json::from_value(value) {
            Ok(folders) => folders,
            Err(e) => {
                log::error!("failed to deserialize folders, skipping migration: {e}");
                return;
            }
        },
        None => Vec::new(),
    };
    let songs: Vec<Song> = match store.get("songs") {
        Some(value) => match serde_json::from_value(value) {
            Ok(songs) => songs,
            Err(e) => {
                log::error!("failed to deserialize songs, skipping migration: {e}");
                return;
            }
        },
        None => Vec::new(),
    };
    let mut normalized: Vec<Song> = Vec::with_capacity(songs.len());

    for mut song in songs {
        if song.folder_id.is_none() {
            song.folder_id = folders
                .iter()
                .filter(|f| path_is_within(&song.path, &f.path))
                .max_by_key(|f| f.path.len())
                .map(|f| f.id.clone());
        }

        let keep = match song.folder_id.as_deref() {
            Some(folder_id) => folders.iter().any(|f| f.id == folder_id),
            // Directly imported songs do not belong to a watched folder.
            None => true,
        };
        if keep {
            normalized.push(song);
        }
    }

    store_set(app, "songs", &normalized);
    store.set("schemaVersion", json!(SCHEMA_VERSION));
}

fn ensure_store_defaults(app: &tauri::AppHandle) {
    if let Ok(store) = app.store(STORE_FILE) {
        if !store.has("songs") {
            store.set("songs", json!([]));
        }
        if !store.has("folders") {
            store.set("folders", json!([]));
        }
        if !store.has("scanHistory") {
            store.set("scanHistory", json!([]));
        }
        if !store.has("playlists") {
            store.set("playlists", json!([]));
        }
        if !store.has("settings") {
            store.set("settings", serde_json::to_value(Settings::default()).unwrap_or(json!({})));
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        // Must be the first plugin registered.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .manage(Arc::new(Mutex::new(DiscordState::new())))
        .plugin(tauri_plugin_updater::Builder::default().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_log::Builder::default().level(log::LevelFilter::Info).build())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            ensure_store_defaults(app.handle());
            migrate_library(app.handle());
            allow_stored_media(app.handle());

            let now_playing_item = IconMenuItem::with_id(app, "now-playing", "Not Playing", false, None, None::<&str>)?;
            let separator1 = PredefinedMenuItem::separator(app)?;
            let prev_item = MenuItem::with_id(app, "prev", "⏮  Previous", true, None::<&str>)?;
            let play_pause_item = MenuItem::with_id(app, "play-pause", "⏯  Play / Pause", true, None::<&str>)?;
            let next_item = MenuItem::with_id(app, "next", "⏭  Next", true, None::<&str>)?;
            let separator2 = PredefinedMenuItem::separator(app)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit lokamusic", true, None::<&str>)?;
            let tray_menu = Menu::with_items(
                app,
                &[&now_playing_item, &separator1, &prev_item, &play_pause_item, &next_item, &separator2, &quit_item],
            )?;
            app.manage(now_playing_item);

            let tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&tray_menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| {
                    match event.id().as_ref() {
                        "quit" => app.exit(0),
                        id @ ("prev" | "play-pause" | "next") => { app.emit("tray:control", id).ok(); }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click {
                        button: tauri::tray::MouseButton::Left,
                        button_state: tauri::tray::MouseButtonState::Up,
                        rect,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("tray-popover") {
                            let _ = w.close();
                        } else {
                            open_tray_popover(app, rect);
                        }
                    }
                })
                .build(app)?;

            app.manage(tray);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            dialog_select_folder,
            dialog_select_files,
            library_get_all_songs,
            library_get_song_art,
            library_get_folders,
            library_add_folder,
            library_remove_folder,
            library_scan_folder,
            library_import_files,
            library_toggle_like,
            library_set_lyrics,
            library_fetch_lyrics,
            library_update_play,
            playlist_get_all,
            playlist_create,
            playlist_delete,
            playlist_rename,
            playlist_add_song,
            playlist_remove_song,
            settings_get,
            settings_set_playback,
            settings_set_analytics,
            settings_set_discord_presence,
            discord_update_presence,
            open_miniplayer,
            close_miniplayer,
            tray_set_now_playing,
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application");

    app.run(|app_handle, event| {
        // Persist any debounced store changes before the process exits.
        if matches!(event, tauri::RunEvent::Exit) {
            save_store_now(app_handle);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::{path_is_within, paths_equal};

    #[test]
    fn path_is_within_matches_components_only() {
        assert!(path_is_within("/music/rock/song.mp3", "/music/rock"));
        assert!(path_is_within("/music/rock", "/music/rock"));
        assert!(!path_is_within("/music/rockabilly/song.mp3", "/music/rock"));
        assert!(!path_is_within("/music", "/music/rock"));
    }

    #[test]
    fn path_is_within_ignores_trailing_separators() {
        assert!(path_is_within("/music/rock/song.mp3", "/music/rock/"));
        assert!(path_is_within("/music/rock/song.mp3", "/music/rock//"));
    }

    #[test]
    fn paths_equal_ignores_trailing_separators() {
        assert!(paths_equal("/music/rock", "/music/rock/"));
        assert!(!paths_equal("/music/rock", "/music/rockabilly"));
    }
}
