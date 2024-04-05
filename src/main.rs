use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
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
    // todo: error handling
    let request = BufReader::new(&mut stream).lines().next().unwrap().unwrap();

    // Resolve client request.
    let (filepath, status_code) = resolve_request(request);
    let content = fs::read_to_string(filepath).unwrap();

    // Respond.
    let response = build_response(status_code, content);
    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

/// Resolves an HTTP request to determine the page path and status code.
///
/// # Returns
///
/// A tuple containing the resolved page path and the corresponding HTTP status code.
// todo: build path from `request` and check (support more page files)
fn resolve_request(request: String) -> (&'static str, HttpStatusCode) {
    let path = request.split_whitespace().nth(1).unwrap();
    match path {
        "/" => ("pages/index.html", HttpStatusCode::Ok),
        _ if path.contains("/") => ("pages/404.html", HttpStatusCode::NotFound),
        _ => ("", HttpStatusCode::BadRequest),
    }
}

fn build_response(status_code: HttpStatusCode, content: String) -> String {
    format!(
        "HTTP/1.1 {}\r\nContent-Length: {}\r\n\r\n{}",
        status_code.status_line(),
        content.len(),
        content
    )
}
