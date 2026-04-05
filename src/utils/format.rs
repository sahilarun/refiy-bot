pub fn formatDuration(milliseconds: u64) -> String {
    let seconds = (milliseconds / 1000) % 60;
    let minutes = (milliseconds / (1000 * 60)) % 60;
    let hours = milliseconds / (1000 * 60 * 60);
    if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        format!("{:02}:{:02}", minutes, seconds)
    }
}
