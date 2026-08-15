use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub enum PlayerRequest {
    Ping,
    GetState,
    SetPlaylist { paths: Vec<PathBuf> },
    Load { path: PathBuf },
    Continue,
    Pause,
    Shutdown,
    Next,
    Prev,
    SetVolume { volume: u32 },
    SetShuffle { enabled: bool },
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PlayerStatus {
    pub is_playing: bool,
    pub volume: u32,
    pub position: f64,
    pub current_path: Option<PathBuf>,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: f64,
    pub shuffle: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerResponse {
    pub error: Option<String>,
    pub state: Option<PlayerStatus>,
}

impl PlayerResponse {
    pub fn ok() -> Self {
        Self {
            error: None,
            state: None,
        }
    }

    pub fn state(state: PlayerStatus) -> Self {
        Self {
            error: None,
            state: Some(state),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            error: Some(message.into()),
            state: None,
        }
    }
}
