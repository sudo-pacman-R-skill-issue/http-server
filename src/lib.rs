use std::{collections::HashMap, io::{BufRead, BufReader, Bytes, Error, ErrorKind, Read, Write}, net::TcpStream};
use bumpalo::Bump;
use http_server_lib::HttpTemplate;

pub struct Request {
    pub method: String,
    pub path: Vec<String>,
    pub protocol: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

impl Request {
    fn new() -> Self {
        Request {
            method: String::with_capacity(4),
            path: Vec::with_capacity(4),
            protocol: String::with_capacity(8),
            headers: HashMap::new(),
            body: None,
        }
    }
    
    fn from(
        method: String,
        path: Vec<String>,
        protocol: String,
        headers: HashMap<String, String>,
    ) -> Self {
        Self {
            method,
            path,
            protocol,
            headers,
            body: None,
        }
    }

    pub fn segment(&self, number: u8) -> Option<&str> {
        self.path.get(number as usize).map(|v| &**v)
    }
 
    pub fn  from_stream(stream: &TcpStream) -> Result<Self, Error> {
        let mut buf_reader = BufReader::new(stream);
        let mut buf_line = String::new();
        buf_reader.read_line(&mut buf_line)?;
        let mut http = buf_line.split_whitespace();
        // dbg!(&http);
        let method = http.next().ok_or_else(|| {
            std::io::Error::new(ErrorKind::InvalidData, "missing method")
        })?;
        let path: Vec<String> = http.next()
            .ok_or_else(|| std::io::Error::new(ErrorKind::InvalidData, "missing path"))
            .unwrap()
            .split("/")
            .map(|e| e.to_string())
            .collect();
        
        let protocol = http.next().ok_or_else(|| {
            std::io::Error::new(ErrorKind::InvalidData, "missing protocol")
        })?;
        // dbg!(&method, &path, &protocol);
        let headers = { 
            http.next().ok_or("");
            let mut headers = HashMap::new();
            loop {
                let mut lline = String::new();
                let bytes = buf_reader.read_line(&mut lline).unwrap_or(0);
                if bytes == 0 || lline == "\r\n" || lline == "\n" {
                    break;
                }
                if let Some((key, value)) = lline.split_once(": ") {
                    headers.insert(key.to_lowercase(), value.to_lowercase());
                }
            }
            headers
        };
        Ok(Request::from(method.to_string(), path, protocol.to_string(), headers))
    }
}

pub enum HttpTemplates {
    PlainText,
    OctetStream,
    Json,
    Created,
    NotFound,
    Slash,
}

impl HttpTemplates {
    #[rustfmt::skip]
    pub fn format(self, content: &str) -> Vec<u8> {
        match self {
            HttpTemplates::PlainText => {
                format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                content.len(),
                content).into_bytes()
            },
            HttpTemplates::OctetStream => {
                format!("HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length:{}\r\n\r\n{}",
                content.len(),
                content).into_bytes()
            },
            HttpTemplates::Json => {
                format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length:{}\r\n\r\n{}",
                content.len(),
                content).into_bytes()},
            HttpTemplates::Created => {
                b"HTTP/1.1 201 Created\r\n\r\n".to_vec()
            },
            HttpTemplates::Slash => {
                b"HTTP/1.1 200 OK\r\n\r\n".to_vec()
            }
            HttpTemplates::NotFound => b"HTTP/1.1 404 Not Found\r\n\r\n".to_vec(),
        }
    }
}

pub struct StatusCode {}
