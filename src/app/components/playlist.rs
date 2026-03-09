use std::path::PathBuf;
use crate::utils::utils;

pub struct PlaylistItem {
    file_path: PathBuf,
    name: String,
    artist: String,
    work: String,
}

impl PlaylistItem {
    pub fn new(file_path: PathBuf, name: String, artist: String, work: String) -> Self {
        Self { file_path, name, artist, work }
    }

    pub fn from_path(file_path: PathBuf) -> Self {
        let file_name = file_path.file_name().and_then(|s| s.to_str()).unwrap_or("Unknown").to_string();
        let (name, artist, work) = utils::parse_file_name(&file_name);
        Self {
            file_path,
            name,
            artist,
            work
        }
    }

    pub fn get_file_path(&self) -> &PathBuf {
        &self.file_path
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_artist(&self) -> &str {
        &self.artist
    }

    pub fn get_work(&self) -> &str {
        &self.work
    }
}

pub struct Playlist {
    pub items: Vec<PlaylistItem>
}

impl Playlist {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn from_paths(file_paths: Vec<PathBuf>) -> Self {
        Self {
            items: file_paths.into_iter().map(PlaylistItem::from_path).collect()
        }
    }
}
