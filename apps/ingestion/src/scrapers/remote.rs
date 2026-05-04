pub fn infer_remote_type(text: &str) -> Option<String> {
    let t = text.to_lowercase();
    if t.contains("remote")
        || t.contains("remotely")
        || t.contains("дистанційно")
        || t.contains("віддален")
    {
        Some("remote".to_string())
    } else if t.contains("hybrid") || t.contains("гібрид") || t.contains("частково") {
        Some("hybrid".to_string())
    } else if t.contains(" office") || t.contains("в офіс") || t.contains("на місці") {
        Some("office".to_string())
    } else {
        None
    }
}
