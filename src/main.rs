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
    // Read the first line from the stream, representing the request.
    let request = match BufReader::new(&mut stream).lines().next() {
        Some(Ok(line)) => line,
        _ => {
            // Respond with 400 if no request exists.
            let error_response = format!(
                "{}\r\nContent-Length: 0\r\n\r\n",
                HttpStatusCode::BadRequest.status_line()
            );
            stream.write_all(error_response.as_bytes()).unwrap();
            stream.flush().unwrap();
            return;
        }
    };

    // Resolve the request.
    let (filepath, status_code) = resolve_request(request);
    let contents = fs::read_to_string(filepath).unwrap();

    // Write response to the stream.
    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_code.status_line(),
        contents.len(),
        contents
    );
    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

/// Resolves an HTTP request to determine the page path and status code.
///
/// # Returns
///
/// A tuple containing the resolved page path and the corresponding HTTP status code.
// todo: add more page files
fn resolve_request(request: String) -> (&'static str, HttpStatusCode) {
    let path = request.split_whitespace().nth(1).unwrap_or("");
    match path {
        "/" => ("pages/index.html", HttpStatusCode::Ok),
        _ => ("pages/404.html", HttpStatusCode::NotFound),
    }
}
