pub fn format_duration(duration: f64) -> String {
    let mut seconds = (duration % 60.0).round() as u64;
    let mut minutes = (duration / 60.0).floor() as u64;
    if seconds == 60 {
        seconds = 0;
        minutes += 1;
    }
    format!("{:02}:{:02}", minutes, seconds)
}
