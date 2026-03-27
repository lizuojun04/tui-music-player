use std::path::PathBuf;

pub struct FileItem {
    pub file_path: PathBuf,
    pub file_name: String
}

impl FileItem {
    /* pub fn new(file_path: PathBuf, file_name: String) -> Self {
        Self { file_path, file_name }
    } */

    pub fn from_path(file_path: PathBuf) -> Self {
        let file_name = file_path.file_name().and_then(|s| s.to_str()).unwrap_or("Unknown").to_string();
        Self {
            file_path,
            file_name
        }
    }

    pub fn get_file_path(&self) -> &PathBuf {
        &self.file_path
    }

    pub fn get_file_name(&self) -> &str {
        &self.file_name
    }

    pub fn is_file(&self) -> bool {
        self.file_path.is_file()
    }

}

pub struct FileBrowser {
    pub items: Vec<FileItem>
}

impl FileBrowser {
    /* pub fn new() -> Self {
        Self { items: Vec::new() }
    } */

    pub fn from_paths(file_paths: Vec<PathBuf>) -> Self {
        let mut items: Vec<FileItem> = file_paths.into_iter().map(FileItem::from_path).collect();
        items.sort_by(|a, b| {
            match (a.is_file(), b.is_file()) {
                (false, false) | (true, true) => {
                    a.get_file_name().to_lowercase().cmp(&b.get_file_name().to_lowercase())
                }, 
                (false, true) => {
                    std::cmp::Ordering::Less
                }, 
                (true, false) => {
                    std::cmp::Ordering::Greater
                }
            }
        });
        Self {
            items
        }
    }
}
