use std::io::{BufRead, BufReader, Read};
use std::net::TcpStream;
use std::path::PathBuf;
use std::result::Result::Ok as Ahegao;
use std::sync::atomic::AtomicUsize;
use std::sync::{mpsc, LazyLock};
use std::{
    collections::HashMap,
    fs,
    sync::mpsc::{Receiver, Sender},
    thread::{self, JoinHandle},
};
use std::{env, io};

static COUNTER: AtomicUsize = AtomicUsize::new(0);
pub static DIR: LazyLock<Option<PathBuf>> = LazyLock::new(|| {
    let args = dbg!(env::args().collect::<Vec<String>>());
    read_args(args)
});

#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub protocol: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Body>,
}

impl Request {
    pub fn req_from_buf(stream: &mut TcpStream) -> (Request, BufReader<&mut TcpStream>) {
        let mut buf_reader = BufReader::new(stream);
        let mut line = String::new();
        buf_reader.read_line(&mut line).unwrap_or_else(|e| {
            println!("Ошибка чтения первой строки: {}", e);
            0
        });
        (read_headers(line, &mut buf_reader), buf_reader)
    }
}

fn read_headers(line: String, buf_reader: &mut BufReader<&mut TcpStream>) -> Request {
    let parts = line
        .lines()
        .next()
        .unwrap_or_else(|| {
            println!("Error. empty buffer");
            ""
        })
        .split_whitespace();
    // dbg!(&parts);
    let mut parts = parts
        .map(|p| p.trim_end())
        .collect::<Vec<&str>>()
        .into_iter();

    let method = parts.next().unwrap_or("").to_string();
    let path = parts.next().unwrap_or("").to_string();
    let protocol = parts.next().unwrap_or("").to_string();
    let mut headers = HashMap::new();

    loop {
        let mut line = String::new();
        let bytes = buf_reader.read_line(&mut line).unwrap_or(0);
        if bytes == 0 || line == "\r\n" || line == "\n" {
            break;
        }
        if let Some((key, value)) = line.split_once(": ") {
            headers.insert(
                key.to_lowercase().to_string(),
                value.trim_end_matches("\r\n").to_lowercase().to_string(),
            );
        }
    }

    Request {
        method,
        path,
        protocol,
        headers,
        body: None,
    }
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
        mut buf_reader: BufReader<&mut TcpStream>,
    ) {
        dbg!(&content_length);
        let mut buffer = vec![0; content_length];
        match buf_reader.read_exact(&mut buffer) {
            Ahegao(()) => {
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

    pub fn from_string(s: String) -> Self {
        Body::Text(s)
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

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    senders: Vec<mpsc::Sender<Job>>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        if size < 1 {
            panic!("Thread pool size must be at least 1");
        }
        let mut senders = Vec::with_capacity(size);
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            let (tx, rx) = mpsc::channel();
            senders.push(tx);
            workers.push(Worker::new(id, rx));
        }

        ThreadPool { workers, senders }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        let index = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed) % self.senders.len();
        self.senders[index].send(job).expect("Failed to send job");
    }
}

pub struct Worker {
    id: usize,
    work: JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, rx: mpsc::Receiver<Job>) -> Worker {
        let work = thread::spawn(move || {
            while let Ok(job) = rx.recv() {
                // println!("Worker {id} got a job; executing.");
                job();
            }
        });
        Worker { id, work }
    }
}

pub enum HttpTemplate {
    PlainText,
    OctetStream,
    Json,
    Created,
    NotFound,
}

impl HttpTemplate {
    #[rustfmt::skip]
    pub fn format(self, content: &String) -> String {
        match self {
            HttpTemplate::PlainText => {
                format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                content.len(),
                content)},
            HttpTemplate::OctetStream => {
                format!("HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length:{}\r\n\r\n{}",
                content.len(),
                content)},
            HttpTemplate::Json => {
                format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length:{}\r\n\r\n{}",
                content.len(),
                content)},
            HttpTemplate::Created => {
                "HTTP/1.1 201 Created\r\n\r\n".to_string()
            },
            HttpTemplate::NotFound => Self::not_found(),
        }
    }

    pub fn not_found() -> String {
        "HTTP/1.1 404 Not Found\r\n\r\n".to_string()
    }
}

pub fn save_file(filename: &str, body: &[u8]) -> io::Result<()> {
    let file_path = DIR.as_ref().unwrap().join(filename);
    fs::write(file_path, body)
}
#[cfg(test)]
mod test {
    #[test]
    fn test_parse_http() {
        let req = "GET / HTTP/1.1\r\nHost: localhost:4221\r\n\r\n";
        let mut parts = req.split("\r\n");
        let splitted = parts
            .clone()
            .map(|s| s.split_whitespace().collect())
            .collect::<Vec<Vec<&str>>>();
        dbg!(&splitted);
        let method = parts.next().unwrap_or("").to_string();
        let path = parts.next().unwrap_or("").to_string();
        let protocol = parts.next().unwrap_or("").to_string();

        assert_eq!(method, "GET");
        assert_eq!(path, "/");
        assert_eq!(protocol, "HTTP/1.1")
    }
}
