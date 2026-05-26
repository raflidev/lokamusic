mod scanner;
mod types;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use urlencoding::encode as url_encode;
use serde_json::json;
use tauri::Emitter;
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

async fn fetch_artwork_url(artist: &str, album: &str) -> String {
    let query = format!("{} {}", artist, album);
    let encoded = url_encode(&query);
    let url = format!(
        "https://itunes.apple.com/search?term={}&entity=album&limit=1",
        encoded
    );

    let Ok(res) = reqwest::get(&url).await else { return "lokamusic_logo".into() };
    let Ok(json) = res.json::<serde_json::Value>().await else { return "lokamusic_logo".into() };

    json["results"][0]["artworkUrl100"]
        .as_str()
        .map(|u| u.replace("100x100bb", "512x512bb"))
        .unwrap_or_else(|| "lokamusic_logo".into())
}

const STORE_FILE: &str = "config.json";

// --- Store helpers ---

fn store_get<T: serde::de::DeserializeOwned>(app: &tauri::AppHandle, key: &str) -> Vec<T> {
    app.store(STORE_FILE)
        .ok()
        .and_then(|s| s.get(key))
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default()
}

fn store_get_obj<T: serde::de::DeserializeOwned + Default>(app: &tauri::AppHandle, key: &str) -> T {
    app.store(STORE_FILE)
        .ok()
        .and_then(|s| s.get(key))
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default()
}

fn store_set<T: serde::Serialize>(app: &tauri::AppHandle, key: &str, value: &T) {
    if let Ok(store) = app.store(STORE_FILE) {
        store.set(key, serde_json::to_value(value).unwrap_or(json!(null)));
        let _ = store.save();
    }
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
    let mut folders: Vec<WatchedFolder> = store_get(&app, "folders");
    if folders.iter().any(|f| f.path == path) {
        return Ok(None);
    }

    let id = format!("{:x}", md5::compute(&path));
    let name = PathBuf::from(&path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(&path)
        .to_string();

    let folder = WatchedFolder {
        id: id.clone(),
        path: path.clone(),
        name,
        song_count: 0,
        size_bytes: 0,
        last_scanned: None,
    };

    folders.push(folder.clone());
    store_set(&app, "folders", &folders);

    let app_clone = app.clone();
    tauri::async_runtime::spawn(async move {
        run_scan(&app_clone, id, path).await;
    });

    Ok(Some(folder))
}

#[tauri::command]
fn library_remove_folder(id: String, app: tauri::AppHandle) -> Result<(), String> {
    let folders: Vec<WatchedFolder> = store_get(&app, "folders");
    let folder = folders.iter().find(|f| f.id == id).cloned();
    let updated_folders: Vec<WatchedFolder> = folders.into_iter().filter(|f| f.id != id).collect();
    store_set(&app, "folders", &updated_folders);

    if let Some(f) = folder {
        let songs: Vec<Song> = store_get::<Song>(&app, "songs")
            .into_iter()
            .filter(|s| !s.path.starts_with(&f.path))
            .collect();
        store_set(&app, "songs", &songs);
        app.emit("library:songs-updated", &songs).ok();
    }

    Ok(())
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
    let mut songs: Vec<Song> = store_get(&app, "songs");
    let existing_ids: std::collections::HashSet<String> = songs.iter().map(|s| s.id.clone()).collect();
    let mut added = 0u32;
    let mut errors = 0u32;

    for path_str in &paths {
        let path = PathBuf::from(path_str);
        match tokio::task::spawn_blocking(move || scanner::read_file_meta(&path)).await {
            Ok(song) if !existing_ids.contains(&song.id) => {
                songs.push(song);
                added += 1;
            }
            Err(_) => errors += 1,
            _ => {}
        }
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

    Ok(ImportResult { added, errors })
}

#[tauri::command]
fn library_toggle_like(song_id: String, app: tauri::AppHandle) -> Result<bool, String> {
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
    Ok(liked)
}

#[tauri::command]
fn library_update_play(song_id: String, app: tauri::AppHandle) -> Result<(), String> {
    let mut songs: Vec<Song> = store_get(&app, "songs");
    for s in &mut songs {
        if s.id == song_id {
            s.play_count += 1;
            s.last_played = Some(now_ms());
            break;
        }
    }
    store_set(&app, "songs", &songs);
    Ok(())
}

#[tauri::command]
fn playlist_get_all(app: tauri::AppHandle) -> Vec<Playlist> {
    store_get(&app, "playlists")
}

#[tauri::command]
fn playlist_create(name: String, app: tauri::AppHandle) -> Result<Playlist, String> {
    let mut playlists: Vec<Playlist> = store_get(&app, "playlists");
    let playlist = Playlist {
        id: uuid::Uuid::new_v4().to_string(),
        name,
        song_ids: vec![],
        created_at: now_ms(),
    };
    playlists.push(playlist.clone());
    store_set(&app, "playlists", &playlists);
    Ok(playlist)
}

#[tauri::command]
fn playlist_delete(id: String, app: tauri::AppHandle) -> Result<(), String> {
    let playlists: Vec<Playlist> = store_get::<Playlist>(&app, "playlists")
        .into_iter()
        .filter(|p| p.id != id)
        .collect();
    store_set(&app, "playlists", &playlists);
    Ok(())
}

#[tauri::command]
fn playlist_rename(id: String, name: String, app: tauri::AppHandle) -> Result<Option<Playlist>, String> {
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
    Ok(found)
}

#[tauri::command]
fn playlist_add_song(playlist_id: String, song_id: String, app: tauri::AppHandle) -> Result<(), String> {
    let mut playlists: Vec<Playlist> = store_get(&app, "playlists");
    for p in &mut playlists {
        if p.id == playlist_id && !p.song_ids.contains(&song_id) {
            p.song_ids.push(song_id);
            break;
        }
    }
    store_set(&app, "playlists", &playlists);
    Ok(())
}

#[tauri::command]
fn playlist_remove_song(playlist_id: String, song_id: String, app: tauri::AppHandle) -> Result<(), String> {
    let mut playlists: Vec<Playlist> = store_get(&app, "playlists");
    for p in &mut playlists {
        if p.id == playlist_id {
            p.song_ids.retain(|id| id != &song_id);
            break;
        }
    }
    store_set(&app, "playlists", &playlists);
    Ok(())
}

#[tauri::command]
fn settings_get(app: tauri::AppHandle) -> Settings {
    store_get_obj(&app, "settings")
}

#[tauri::command]
async fn settings_set_discord_presence(
    enabled: bool,
    app: tauri::AppHandle,
    discord: tauri::State<'_, Arc<Mutex<DiscordState>>>,
) -> Result<(), String> {
    let mut settings: Settings = store_get_obj(&app, "settings");
    settings.discord_presence = enabled;
    store_set(&app, "settings", &settings);

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
                            d.artwork_cache.insert(cache_key, url.clone());
                        }
                        url
                    }
                }
            };

            DiscordAction::SetActivity {
                state_str: format!("{} — {}", artist, album),
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
        let files = scanner::collect_audio_files(&path);
        let total = files.len() as u32;
        let mut songs: Vec<Song> = Vec::with_capacity(files.len());
        let errors = 0u32;

        for (i, file) in files.iter().enumerate() {
            songs.push(scanner::read_file_meta(file));
            let scanned = (i + 1) as u32;
            let percent = scanned * 100 / total.max(1);
            let _ = tx.blocking_send((scanned, total, percent));
        }

        (songs, errors)
    });

    while let Some((scanned, total, percent)) = rx.recv().await {
        app.emit("library:scan-progress", ScanProgressPayload {
            folder_id: folder_id.clone(),
            scanned,
            total,
            percent,
        }).ok();
    }

    let (new_songs, errors) = match scan_handle.await {
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

    // Merge: preserve liked/playCount/lastPlayed from existing songs
    let existing: Vec<Song> = store_get(app, "songs");
    let existing_map: HashMap<String, Song> = existing.into_iter().map(|s| (s.id.clone(), s)).collect();

    let kept: Vec<Song> = existing_map.values()
        .filter(|s| !s.path.starts_with(&folder_path))
        .cloned()
        .collect();

    let updated: Vec<Song> = new_songs.into_iter().map(|mut s| {
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

    let size_bytes = scanner::get_folder_size(&PathBuf::from(&folder_path));
    let mut folders: Vec<WatchedFolder> = store_get(app, "folders");
    let song_count = merged.iter().filter(|s| s.path.starts_with(&folder_path)).count() as u32;
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

    app.emit("library:scan-complete", ScanCompletePayload {
        folder_id: folder_id_for_scan,
        songs: merged,
        folders,
    }).ok();
}

// --- App setup ---

fn purge_orphan_songs(app: &tauri::AppHandle) {
    let folders: Vec<WatchedFolder> = store_get(app, "folders");
    let songs: Vec<Song> = store_get::<Song>(app, "songs")
        .into_iter()
        .filter(|s| folders.iter().any(|f| s.path.starts_with(&f.path)))
        .collect();
    store_set(app, "songs", &songs);
}

fn ensure_store_defaults(app: &tauri::AppHandle) {
    if let Ok(store) = app.store(STORE_FILE) {
        if store.get("songs").is_none() {
            store.set("songs", json!([]));
        }
        if store.get("folders").is_none() {
            store.set("folders", json!([]));
        }
        if store.get("scanHistory").is_none() {
            store.set("scanHistory", json!([]));
        }
        if store.get("playlists").is_none() {
            store.set("playlists", json!([]));
        }
        if store.get("settings").is_none() {
            store.set("settings", serde_json::to_value(Settings::default()).unwrap_or(json!({})));
        }
        let _ = store.save();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Arc::new(Mutex::new(DiscordState::new())))
        .plugin(tauri_plugin_log::Builder::default().level(log::LevelFilter::Info).build())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            ensure_store_defaults(app.handle());
            purge_orphan_songs(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            dialog_select_folder,
            dialog_select_files,
            library_get_all_songs,
            library_get_folders,
            library_add_folder,
            library_remove_folder,
            library_scan_folder,
            library_import_files,
            library_toggle_like,
            library_update_play,
            playlist_get_all,
            playlist_create,
            playlist_delete,
            playlist_rename,
            playlist_add_song,
            playlist_remove_song,
            settings_get,
            settings_set_discord_presence,
            discord_update_presence,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
