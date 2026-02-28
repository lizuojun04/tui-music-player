use std::path::PathBuf;

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
        let (name, artist, work) = Self::parse_file_name(&file_name);
        Self {
            file_path,
            name,
            artist,
            work
        }
    }

    fn parse_file_name(filename: &str) -> (String, String, String) {
        let stem = if let Some(dot_index) = filename.rfind('.') {
            &filename[..dot_index]
        } else {
            filename
        };

        let (left_part, artist) = match stem.rfind(" - ") {
            Some(separate_index) => {
                let (left, right) = stem.split_at(separate_index);
                (left.trim(), right.strip_prefix(" - ").           
                     unwrap_or(right).trim().to_string()) 
            },
            None => (stem.trim(), "Unknown".to_string())
        };

        let (name, work) = match left_part.rfind('(') {
            Some(open_paren_index) => {
                if let Some(close_paren_index) = left_part[open_paren_index..].find(')') {
                    let close_paren_index = open_paren_index + close_paren_index;
                    let name = left_part[..open_paren_index].trim().to_string();
                    let work = left_part[open_paren_index + 1..close_paren_index].trim().to_string();
                    (name, work)
                } else {
                    (left_part.trim().to_string(), "Unknown".to_string())
                }
            },
            None => (left_part.trim().to_string(), "Unknown".to_string())
        };
        (name, artist, work)
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
