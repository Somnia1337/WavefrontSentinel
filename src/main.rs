use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};
use wavefront_sentinel::ThreadPool;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:80").unwrap();
    let pool = ThreadPool::new(10);
    for stream in listener.incoming().map(|s| s.unwrap()) {
        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

/// 处理一个 TCP 流
///
/// 解析请求, 选择响应的信息和文件, 并进行响应
fn handle_connection(mut stream: TcpStream) {
    // 读取请求的第 1 行
    let reader = BufReader::new(&mut stream);
    let request = reader.lines().next().unwrap().unwrap();

    // 选择响应的信息和文件
    let (status_line, filename) = if request == "GET / HTTP/1.1" {
        ("HTTP/1.1 200 OK", "pages\\hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "pages\\404.html")
    };

    // 响应
    let contents = fs::read_to_string(filename).unwrap();
    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );
    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
