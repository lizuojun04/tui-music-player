pub fn format_duration(duration: f64) -> String {
    let mut seconds = (duration % 60.0).round() as u64;
    let mut minutes = (duration / 60.0).floor() as u64;
    if seconds == 60 {
        seconds = 0;
        minutes += 1;
    }
    format!("{:02}:{:02}", minutes, seconds)
}


pub fn parse_file_name(filename: &str) -> (String, String, String) {
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
