use std::path::PathBuf;
use walkdir::WalkDir;

pub struct FileManager {
    root_dir: PathBuf
}

impl FileManager {
    pub fn new(root_dir: PathBuf) -> Self {
        Self { root_dir }
    }

    pub fn get_file_path_list(&self) -> Vec<PathBuf> {
        WalkDir::new(&self.root_dir).follow_links(true)
                                          .into_iter()
                                          .filter_map(|entry| entry.ok())
                                          .map(|entry| entry.path().to_path_buf())
                                          .collect::<Vec<PathBuf>>()
    }

    pub fn get_file_path_list_static(root_dir: PathBuf) -> Vec<PathBuf> {
        WalkDir::new(root_dir).follow_links(true)
                              .into_iter()
                              .filter_map(|entry| entry.ok())
                              .filter_map(|file| if file.file_type().is_file() { Some(file) } else { None })
                              .map(|entry| entry.path().to_path_buf())
                              .collect::<Vec<PathBuf>>()
    }
}
