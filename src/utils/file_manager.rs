use std::{fs::DirEntry, path::PathBuf};
use walkdir::WalkDir;

pub struct FileManager {
    root_dir: PathBuf
}

impl FileManager {
    pub fn new(root_dir: PathBuf) -> Self {
        Self { root_dir }
    }

    pub fn get_file_path_list(&self) -> Vec<PathBuf> {
        WalkDir::new(&self.root_dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|entry| {
                match entry.ok() {
                    Some(file) if file.file_type().is_file() => Some(file),
                    _ => None
                }
            })
            .map(|entry| entry.path().to_path_buf())
            .filter_map(|path_buf| if Self::is_supported_file(&path_buf) {
                Some(path_buf)
            } else {
                None
            })
            .collect::<Vec<PathBuf>>()
    }

    pub fn get_file_path_list_static(root_dir: PathBuf) -> Vec<PathBuf> {
        WalkDir::new(root_dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|entry| {
                match entry.ok() {
                    Some(file) if file.file_type().is_file() => Some(file),
                    _ => None
                }
            })
            .map(|entry| entry.path().to_path_buf())
            .filter_map(|path_buf| {
                if Self::is_supported_file(&path_buf) {
                    Some(path_buf)
                } else {
                    None
                }
            })
            .collect::<Vec<PathBuf>>()
    }

    fn get_entry_list(&self) -> Vec<PathBuf> {
        WalkDir::new(&self.root_dir)
            .follow_links(true)
            .max_depth(1)
            .into_iter()
            .filter_entry(|entry| {
                !entry.file_name()
                .to_str()
                .map(|s| s.starts_with("."))
                .unwrap_or(false)
            })
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path().to_path_buf())
            .filter_map(|path_buf| {
                if path_buf.is_file() && !Self::is_supported_file(&path_buf) {
                    None
                } else {
                    Some(path_buf)
                }
            })
            .skip(1)
            .collect::<Vec<PathBuf>>()
    }

    pub fn get_entry_list_static(root_dir: PathBuf) -> Vec<PathBuf> {
        WalkDir::new(root_dir)
            .follow_links(true)
            .max_depth(1)
            .into_iter()
            .filter_entry(|entry| {
                !entry.file_name()
                .to_str()
                .map(|s| s.starts_with("."))
                .unwrap_or(false)
            })
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path().to_path_buf())
            .filter_map(|path_buf| {
                if path_buf.is_file() && !Self::is_supported_file(&path_buf) {
                    None
                } else {
                    Some(path_buf)
                }
            })
            .skip(1)
            .collect::<Vec<PathBuf>>()
    }

    fn is_supported_file(path_buf: &PathBuf) -> bool {
        // TODO 这里硬编码了
        if let Some(extension) = path_buf.extension() {
            matches!(extension.to_str().unwrap_or("").to_lowercase().as_str(), "mp4" | "mp3" | "flac" | "wav" | "ogg")
        } else {
            false
        }
    }
}
