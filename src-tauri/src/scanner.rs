use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use lofty::prelude::*;
use lofty::probe::Probe;
use lofty::picture::PictureType;
use lofty::tag::ItemKey;
use walkdir::WalkDir;
use crate::types::Song;

const AUDIO_EXTENSIONS: &[&str] = &["mp3", "flac", "wav", "ogg", "m4a", "aac", "wma", "opus"];
const MAX_ART_BYTES: usize = 5 * 1024 * 1024;
const MAX_ART_DIMENSION: u32 = 512;

/// Collects audio files and the total size of every file in the directory in
/// one walk (avoids walking the tree a second time just to compute the size).
pub fn collect_audio_files(dir: &Path) -> (Vec<PathBuf>, u64) {
    let mut files = Vec::new();
    let mut size_bytes = 0u64;

    for entry in WalkDir::new(dir).follow_links(false).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        if let Ok(metadata) = entry.metadata() {
            size_bytes += metadata.len();
        }
        let is_audio = entry
            .path()
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| AUDIO_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
            .unwrap_or(false);
        if is_audio {
            files.push(entry.path().to_path_buf());
        }
    }

    (files, size_bytes)
}

pub fn read_file_meta(file_path: &Path) -> Song {
    let path_str = file_path.to_string_lossy().replace('\\', "/");
    let id = format!("{:x}", md5::compute(&path_str));

    let file_stem = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();

    let mut title = file_stem;
    let mut artist = "Unknown Artist".to_string();
    let mut album = "Unknown Album".to_string();
    let mut duration = 0.0f64;
    let mut lyrics: Option<String> = None;

    let date_added = fs::metadata(file_path)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as f64)
        .unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as f64
        });

    if let Ok(tagged_file) = Probe::open(file_path).and_then(|p| p.read()) {
        duration = tagged_file.properties().duration().as_secs_f64();

        let tag = tagged_file.primary_tag().or_else(|| tagged_file.first_tag());
        if let Some(tag) = tag {
            if let Some(t) = tag.title() {
                title = t.into_owned();
            }
            if let Some(a) = tag.artist() {
                artist = a.into_owned();
            }
            if let Some(a) = tag.album() {
                album = a.into_owned();
            }

            if let Some(l) = tag.get_string(&ItemKey::Lyrics) {
                let trimmed = l.trim().to_string();
                if !trimmed.is_empty() {
                    lyrics = Some(trimmed);
                }
            }
        }
    }

    // Prefer .lrc sidecar over embedded lyrics
    let lrc_path = file_path.with_extension("lrc");
    if lrc_path.exists() {
        if let Ok(content) = fs::read_to_string(&lrc_path) {
            let trimmed = content.trim().to_string();
            if !trimmed.is_empty() {
                lyrics = Some(trimmed);
            }
        }
    }

    Song {
        id,
        path: path_str,
        folder_id: None,
        title,
        artist,
        album,
        duration,
        lyrics,
        date_added,
        liked: false,
        play_count: 0,
        last_played: None,
    }
}

pub fn read_song_art(file_path: &Path) -> Option<String> {
    let tagged_file = Probe::open(file_path).and_then(|p| p.read()).ok()?;
    let tag = tagged_file.primary_tag().or_else(|| tagged_file.first_tag())?;
    let cover = tag
        .pictures()
        .iter()
        .find(|p| p.pic_type() == PictureType::CoverFront)
        .or_else(|| tag.pictures().first())?;
    if cover.data().len() > MAX_ART_BYTES {
        return None;
    }

    // Downscale covers before sending them to the webview: full-size art can be
    // several megabytes of base64 per song, which is far more than the UI needs.
    if let Ok(image) = image::load_from_memory(cover.data()) {
        let thumbnail = image.thumbnail(MAX_ART_DIMENSION, MAX_ART_DIMENSION);
        let mut buffer = Cursor::new(Vec::new());
        if thumbnail.write_to(&mut buffer, image::ImageFormat::Jpeg).is_ok() {
            return Some(format!(
                "data:image/jpeg;base64,{}",
                BASE64.encode(buffer.into_inner())
            ));
        }
    }

    let mime = cover.mime_type().map(|m| m.to_string()).unwrap_or_else(|| "image/jpeg".to_string());
    Some(format!("data:{};base64,{}", mime, BASE64.encode(cover.data())))
}
