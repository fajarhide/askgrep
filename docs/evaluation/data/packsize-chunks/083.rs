fn serve(mut stream: TcpStream, store: &Store) -> Result<()> {
    let mut line = String::new();
    BufReader::new(&stream).read_line(&mut line)?;
    let path = request_path(&line);

    let (status, body) = match path.as_str() {
        "/" => ("200 OK", page(store)),
        // Enough to tell "the server is up" from "the page is broken" without
        // rendering anything.
        "/health" => ("200 OK", "ok".to_string()),
        _ => ("404 Not Found", "not found".to_string()),
    };

    let content_type = if path == "/" {
        "text/html; charset=utf-8"
    } else {
        "text/plain; charset=utf-8"
    };
    write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )?;
    stream.flush()?;
    Ok(())
}

/// The path from a request line, or `/` when it is not one we understand.
///
/// Only `GET` is answered. A dashboard that reads a database has no reason to
/// accept anything that writes.