use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    path::PathBuf,
};
use wavefront_sentinel::{HttpStatusCode, ThreadPool};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:80").unwrap();
    let pool = ThreadPool::new(10);
    for stream in listener.incoming().map(|s| s.unwrap()) {
        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

/// Handles a TCP stream by parsing the request and responding to it.
fn handle_connection(mut stream: TcpStream) {
    // Read the first line from the stream, representing client request.
    let reader = BufReader::new(&mut stream);
    let response = match reader.lines().next().and_then(|line| line.ok()) {
        None => build_response(HttpStatusCode::BadRequest, String::new()),
        Some(request) => {
            // Resolve client request.
            let (filepath, status_code) = resolve_request(&request);
            let content = fs::read_to_string(filepath).unwrap();
            build_response(status_code, content)
        }
    };
    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

/// Resolves an HTTP request.
///
/// Returns a tuple, containing the path to the requested page and the corresponding `HttpStatusCode`.
fn resolve_request(request: &str) -> (PathBuf, HttpStatusCode) {
    match build_path(request) {
        Ok(path) => {
            if path.exists() {
                (path, HttpStatusCode::Ok)
            } else {
                (PathBuf::from("pages/404.html"), HttpStatusCode::NotFound)
            }
        }
        Err(_) => (PathBuf::from("pages/400.html"), HttpStatusCode::BadRequest),
    }
}

/// Builds a path to the requested page.
///
/// # Errors
///
/// Returns an `Err` if the HTTP request does not contain a path argument.
fn build_path(request: &str) -> Result<PathBuf, ()> {
    let mut path = PathBuf::from("pages");
    match request.split_whitespace().nth(1) {
        Some(request_path) if !request_path.is_empty() => {
            let mut page_path = request_path.to_string();
            if page_path == "/" {
                path.push("index.html");
            } else {
                if !page_path.ends_with(".html") {
                    page_path.push_str(".html");
                }
                path.push(page_path.trim_start_matches('/'));
            }
            Ok(path)
        }
        _ => Err(()),
    }
}

/// Builds an HTTP response message.
fn build_response(status_code: HttpStatusCode, content: String) -> String {
    format!(
        "HTTP/1.1 {}\r\nContent-Length: {}\r\n\r\n{}",
        status_code.status_line(),
        content.len(),
        content
    )
}
