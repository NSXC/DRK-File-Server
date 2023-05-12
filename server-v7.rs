use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{Write};
use std::net::{TcpListener};
use std::path::PathBuf;
use std::fs::File;
use serde_json::Value;
use std::io::Read;

fn main() {
    let mut configfile = File::open("drk.config.json").expect("Unable to open file");
    let mut configcontents = String::new();
    configfile.read_to_string(&mut configcontents).expect("Unable to read file");
    let  config: Value = serde_json::from_str(&configcontents).expect("Unable to parse JSON");
    let port = &config["port"];
    let address = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(address).unwrap();
    println!("Listening for connections on port {}...",port);

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let mut buffer: [u8; 1024] = [0; 1024];//max amount of bytes to read
        stream.read(&mut buffer).unwrap();
        let request = String::from_utf8_lossy(&buffer[..]);

        let response = handle_request(request.to_string());

        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
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
    //let body = parts[1].trim();
    let mut message = String::new();

    let mut path = "/";
    for line in request.lines() {
        if line.starts_with("GET") {
            let request_parts: Vec<&str> = line.split_whitespace().collect();
            if request_parts.len() >= 2 {
                path = request_parts[1];
                let mut configfile = File::open("drk.config.json").expect("Unable to open file");
                let mut configcontents = String::new();
                configfile.read_to_string(&mut configcontents).expect("Unable to read file");
                let config: Value = serde_json::from_str(&configcontents).expect("Unable to parse JSON");
                if let Some(blocked) = config["blocked"].as_array() {
                    if blocked.contains(&Value::String(path.to_owned())) {
                        message += "HTTP/1.1 403 Not Allowed\r\n";
                        message += "Content-Type: text/plain\r\n";
                        message += "\r\n";
                        message += "403 File Not Allowed";
                        return Some(message);
                    }
                }
                break;
            }
        }
    }


    let routing_table = read_routing_table();
    if let Some(file_path) = routing_table.get(path) {
        let file_path = PathBuf::from(file_path);
        let content_type = match file_path.extension().and_then(|ext| ext.to_str()) {
            Some("html") => "text/html",
            Some("drk") => "text/html",
            Some("css") => "text/css",
            Some("js") => "text/javascript",
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("png") => "image/png",
            Some("gif") => "image/gif",
            Some("ico") => "image/vnd.microsoft.icon",
            Some("json") => "application/json",
            Some("txt") => "text/plain",
            _ => "application/octet-stream", // default content type for other file types
        };
        let file_data = fs::read(file_path.as_path()).unwrap_or_else(|_| {
            String::from(r#"404 Not Found"#).into()
        });

        message += "HTTP/1.1 200 OK\r\n";
        message += &format!("Content-Type: {}\r\n", content_type);
        message += "\r\n";
        message += &String::from_utf8_lossy(&file_data);

        return Some(message);
    }

    let file_path = PathBuf::from(&path[1..]);
    let content_type = match file_path.extension().and_then(|ext| ext.to_str()) {
        Some("html") => "text/html",
        Some("css") => "text/css",
        Some("js") => "text/javascript",
        Some("drk") => "text/html",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("ico") => "image/vnd.microsoft.icon",
        Some("json") => "application/json",
        Some("txt") => "text/plain",
        _ => "application/octet-stream",  // default content type for other file types
    };

    if let Ok(file_data) = fs::read(file_path.clone()) {
        message += "HTTP/1.1 200 OK\r\n";
        message += &format!("Content-Type: {}\r\n", content_type);
        message += "\r\n";
        message += &String::from_utf8_lossy(&file_data);
    } else {
        let mut found = false;
        for entry in fs::read_dir(".").ok()? {
            if let Ok(entry) = entry {
                if entry.path().is_dir() {
                    let sub_path = entry.path().join(file_path.clone());
                    if let Ok(file_data) = fs::read(sub_path.clone()) {
                        found = true;
                        message += "HTTP/1.1 200 OK\r\n";
                        message += &format!("Content-Type: {}\r\n", content_type);
                        message += "\r\n";
                        message += &String::from_utf8_lossy(&file_data);
                        break;
                    }
                }
            }
        }
        if !found {
            message += "HTTP/1.1 404 Not Found\r\n";
            message += "Content-Type: text/plain\r\n";
            message += "\r\n";
            message += "404 Not Found";
        }
    }

    Some(message)
}
#[derive(Serialize, Deserialize)]
struct RoutingTable {
    routes: HashMap<String, String>,
}
fn read_routing_table() -> HashMap<String, String> {
    let routing_file = fs::read_to_string("drk.config.json").unwrap();
    let routing_table: RoutingTable = serde_json::from_str(&routing_file).unwrap();
    routing_table.routes
}
