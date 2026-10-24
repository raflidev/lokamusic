use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Song {
    pub id: String,
    pub path: String,
    /// Watched folder the song belongs to. `None` for directly imported songs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub folder_id: Option<String>,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lyrics: Option<String>,
    pub date_added: f64,
    pub liked: bool,
    pub play_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_played: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchedFolder {
    pub id: String,
    pub path: String,
    pub name: String,
    pub song_count: u32,
    pub size_bytes: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_scanned: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanEvent {
    pub timestamp: f64,
    pub folder_path: String,
    pub success: u32,
    pub errors: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Playlist {
    pub id: String,
    pub name: String,
    pub song_ids: Vec<String>,
    pub created_at: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    #[serde(default = "default_volume")]
    pub volume: f64,
    #[serde(default)]
    pub shuffle: bool,
    #[serde(default = "default_repeat")]
    pub repeat: String,
    #[serde(default = "default_true")]
    pub discord_presence: bool,
    #[serde(default = "default_true")]
    pub analytics: bool,
}

fn default_volume() -> f64 {
    0.8
}

fn default_repeat() -> String {
    "none".to_string()
}

fn default_true() -> bool {
    true
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            volume: 0.8,
            shuffle: false,
            repeat: "none".to_string(),
            discord_presence: true,
            analytics: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanProgressPayload {
    pub folder_id: String,
    pub scanned: u32,
    pub total: u32,
    pub percent: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanCompletePayload {
    pub folder_id: String,
    pub songs: Vec<Song>,
    pub folders: Vec<WatchedFolder>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub added: u32,
    pub errors: u32,
}
