fn request_path(request_line: &str) -> String {
    let mut parts = request_line.split_whitespace();
    match (parts.next(), parts.next()) {
        (Some("GET"), Some(path)) => path.split('?').next().unwrap_or("/").to_string(),
        _ => "/nothing".to_string(),
    }
}

/// Minimal escaping for the few values that reach the page from the database.