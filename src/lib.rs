use std::{
    collections::HashMap,
    env, fs,
    io::{BufRead, BufReader, Error, ErrorKind, Read},
    net::TcpStream,
    path::PathBuf,
    sync::LazyLock,
};

pub static DIR: LazyLock<Option<PathBuf>> = LazyLock::new(|| {
    let args = dbg!(env::args().collect::<Vec<String>>());
    read_args(args)
});

#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub path: Vec<String>,
    pub protocol: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Body>,
}

impl Request {
    fn new() -> Self {
        Request {
            method: String::with_capacity(4),
            path: Vec::with_capacity(4),
            protocol: String::with_capacity(8),
            headers: HashMap::new(),
            body: Some(Body::Text("".to_string())),
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
            body: Some(Body::Text("".to_string())),
        }
    }

    pub fn segment(&self, number: u8) -> Option<&str> {
        self.path.get(number as usize).map(|v| &**v)
    }

    pub fn from_stream(buf_reader: &mut BufReader<TcpStream>) -> Result<Self, Error> {
        let mut buf_line = String::new();
        buf_reader.read_line(&mut buf_line)?;
        let mut http = buf_line.split_whitespace();
        // dbg!(&http);
        let method = http
            .next()
            .ok_or_else(|| std::io::Error::new(ErrorKind::InvalidData, "missing method"))?;
        let path: Vec<String> = http
            .next()
            .ok_or_else(|| std::io::Error::new(ErrorKind::InvalidData, "missing path"))
            .unwrap()
            .split("/")
            .map(|e| e.to_string())
            .collect();

        let protocol = http
            .next()
            .ok_or_else(|| std::io::Error::new(ErrorKind::InvalidData, "missing protocol"))?;
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
        Ok(Request::from(
            method.to_string(),
            path,
            protocol.to_string(),
            headers,
        ))
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

pub fn read_args(args: Vec<String>) -> Option<std::path::PathBuf> {
    if args.len() < 3 {
        println!(
            "
        NO FLAGS *vine boom*. 
        NO BALLS*vine boom*. 
        AND PROBABLY NO BUTTHOLE SINCE THIS GUY FEEDS ON RADIATION*boosted vine boom*
        "
        );
        None
    } else if args[1] == "--directory" {
        Some(std::path::PathBuf::from(&args[2]))
    } else {
        None
    }
}

pub fn save_file(filename: &str, body: &[u8]) -> std::io::Result<()> {
    let file_path = DIR.as_ref().unwrap().join(filename);
    fs::write(file_path, body)
}

#[derive(Debug)]
pub enum Body {
    Text(String),
    Binary(Vec<u8>),
}
impl Body {
    pub fn read_body(
        body: &mut Option<Body>,
        content_length: usize,
        buf_reader: &mut BufReader<TcpStream>,
    ) {
        dbg!(&content_length);
        let mut buffer = vec![0; content_length];
        match buf_reader.read_exact(&mut buffer) {
            Ok(()) => {
                *body = Some(Body::from_bytes(buffer));
            }
            Err(e) => {
                println!("Ошибка чтения тела: {e}");
                *body = Some(Body::Binary(Vec::new()));
            }
        }
    }

    pub fn as_text(&self) -> Option<&str> {
        match self {
            Body::Text(s) => Some(s),
            Body::Binary(_) => None,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Body::Text(s) => s.as_bytes(),
            Body::Binary(b) => b,
        }
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        match String::from_utf8(bytes.clone()) {
            Ok(s) => Body::Text(s),
            Err(_) => Body::Binary(bytes),
        }
    }
}
