use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::thread;
use std::io::Read;
fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

    println!("Listening for connections on port 8080...");

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        println!("Connection established!");

        let mut buffer: [u8; 1024] = [0; 1024];//max amount of bytes to read
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
    if let Some(message) = get_message(request.to_string()) {
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

    let routing_table = read_routing_table();

    // Try to find the file in the routing table
    if let Some(file_path) = routing_table.get(path) {
        let drk_file_path = PathBuf::from(file_path);
        let content_type = if file_path.ends_with(".drk") {
            "text/html"
        } else if file_path.ends_with(".css") {
            "text/css"
        } else if file_path.ends_with(".js") {
            "text/javascript"
        } else {
            "text/html"
        };
        let drk_file = fs::read_to_string(drk_file_path.as_path()).unwrap_or_else(|_| {
            String::from(
                r#"404-Z DRK FILE NOT FOUND"#,
            )
        });

        message += "HTTP/1.1 200 OK\r\n";
        message += &format!("Content-Type: {}\r\n", content_type);
        message += "\r\n";
        message += &drk_file;

        return Some(message);
    }

    // Try to find the file outside the routing table
    let file_path = PathBuf::from(&path[1..]);
    let content_type = if path.ends_with(".html") {
        "text/html"
    } else if path.ends_with(".css") {
        "text/css"
    } else if path.ends_with(".js") {
        "text/javascript"
    } else {
        "text/html"
    };

    if let Ok(file_data) = fs::read(file_path) {
        message += "HTTP/1.1 200 OK\r\n";
        message += &format!("Content-Type: {}\r\n", content_type);
        message += "\r\n";
        message += &String::from_utf8_lossy(&file_data);
    } else {
        message += "HTTP/1.1 404 Not Found\r\n";
        message += "Content-Type: text/html\r\n";
        message += "\r\n";
        message += "404 Not Found";
    }

    Some(message)
}



#[derive(Serialize, Deserialize)]
struct RoutingTable {
    routes: HashMap<String, String>,
}

fn read_routing_table() -> HashMap<String, String> {
    let routing_file = fs::read_to_string("drk.routes.json").unwrap();
    let routing_table: RoutingTable = serde_json::from_str(&routing_file).unwrap();
    routing_table.routes
}
