use std::fs;
use std::io::{self, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::io::Read;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

    println!("Listening for connections on port 8080...");

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        println!("Connection established!");

        let mut buffer = [0; 1024];
        stream.read(&mut buffer).unwrap();
        let request = String::from_utf8_lossy(&buffer[..]);

        let response = handle_request(request.to_string());

        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();

        println!("Response sent!");
    }
}

fn handle_request(request: String) -> String {
    let mut response = String::new();
    if let Some(message) = get_message(request) {
        response += &message;
    } else {
        response += "HTTP/1.1 400 Bad Request\r\n";
    }
    response
}

fn get_message(request: String) -> Option<String> {
    let parts: Vec<&str> = request.splitn(2, "\r\n\r\n").collect();
    if parts.len() < 2 {
        return None;
    }
    let body = parts[1].trim();
    let mut message = String::new();

    // Check if the request is for index.drk
    let mut path = "/";
    for line in request.lines() {
        if line.starts_with("GET") {
            let request_parts: Vec<&str> = line.split_whitespace().collect();
            if request_parts.len() >= 2 {
                path = request_parts[1];
                break;
            }
        }
    }

    // Serve the index.drk file with DRK code
    if path == "/" || path.ends_with("/index.drk") {
        let drk_file_path = format!(".{}", path);
        let content_type = if path.ends_with(".drk") {
            "text/html" // treat DRK files as HTML files
        } else {
            "text/html"
        };
        let drk_file = fs::read_to_string(drk_file_path).unwrap_or_else(|_| {
            String::from(
                r#"<!DOCTYPE html>
<html>
  <head>
    <title>DRK Page Not Found</title>
  </head>
  <body>
    <h1>DRK Page Not Found</h1>
  </body>
</html>"#,
            )
        });
        message += "HTTP/1.1 200 OK\r\n";
        message += &format!("Content-Type: {}\r\n", content_type);
        message += "\r\n";
        message += &drk_file;
    } else {
        message += "HTTP/1.1 404 Not Found\r\n";
        message += "Content-Type: text/html\r\n";
        message += "\r\n";
        message += "404 Not Found";
    }

    Some(message)
}
