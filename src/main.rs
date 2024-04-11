use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    path::PathBuf,
};
use wavefront_sentinel::HttpContentType::Html;
use wavefront_sentinel::{HttpContentType, HttpStatusCode, ThreadPool};

fn main() {
    println!("> Starting up...");
    let listener = TcpListener::bind("127.0.0.1:80").unwrap();
    let pool = ThreadPool::new(10);
    println!("> Started running.");
    for stream in listener.incoming().map(|s| s.unwrap()) {
        pool.execute(|| {
            handle_connection(stream);
        });
    }

    // todo: elegant shutdown
}

/// Handles a TCP stream by parsing the request and responding to it.
fn handle_connection(mut stream: TcpStream) {
    // Read the first line from the stream, representing request.
    let reader = BufReader::new(&mut stream);
    let response = match reader.lines().next().and_then(|line| line.ok()) {
        None => build_response(HttpStatusCode::BadRequest, vec![], Html),
        Some(request) => {
            // Resolve the request.
            let (filepath, status_code, content_type) = resolve_request(&request);
            let content = fs::read(filepath).unwrap();
            build_response(status_code, content, content_type)
        }
    };
    stream.write_all(&response).unwrap();
    stream.flush().unwrap();
}

/// Resolves an HTTP request.
///
/// # Returns
///
/// A tuple, containing a `PathBuf` to the requested page and the corresponding `HttpStatusCode`.
fn resolve_request(request: &str) -> (PathBuf, HttpStatusCode, HttpContentType) {
    match build_path_to_page(request) {
        Ok(path) => {
            if path.exists() {
                let extension = path.extension().unwrap();
                (
                    path.clone(),
                    HttpStatusCode::Ok,
                    HttpContentType::from(extension.to_str().unwrap()),
                )
            } else {
                (
                    PathBuf::from("pages/404.html"),
                    HttpStatusCode::NotFound,
                    Html,
                )
            }
        }
        Err(_) => (
            PathBuf::from("pages/400.html"),
            HttpStatusCode::BadRequest,
            Html,
        ),
    }
}

/// Builds a path to the requested page.
///
/// # Returns
///
/// - A `PathBuf` containing the requested page if the process succeeds.
/// - An `Err` if fails.
fn build_path_to_page(request: &str) -> Result<PathBuf, ()> {
    let mut path = PathBuf::from("pages");
    match request.split_whitespace().nth(1) {
        Some(request_path) if !request_path.is_empty() => {
            let mut page_path = request_path.to_string();
            if page_path == "/" {
                path.push("index.html");
            } else {
                if !page_path.contains(".") {
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
fn build_response(
    status_code: HttpStatusCode,
    content: Vec<u8>,
    content_type: HttpContentType,
) -> Vec<u8> {
    let content_length = content.len();
    let content_type = content_type.content_type();
    let status_line = status_code.status_line();
    let response = format!(
        "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
        status_line, content_type, content_length
    );
    let mut response_bytes = response.into_bytes();
    response_bytes.extend_from_slice(&content);
    response_bytes
}
