use http_server_lib::{
    Body, HttpTemplate, Request, ThreadPool, DIR,
};
use std::{
    fs,
    io::prelude::*,
    net::{TcpListener, TcpStream},
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    let pool = ThreadPool::new(64);
    for stream in listener.incoming() {
        match stream {
            std::result::Result::Ok(stream) => {
                pool.execute(move || {
                    dvuhsotaya(stream).unwrap_or_else(|e| {
                        eprintln!("{e}");
                    });
                });
            }
            Err(e) => {
                eprintln!("{e}")
            }
        }
    }
}

fn dvuhsotaya(mut stream: TcpStream) -> anyhow::Result<()> {
    loop {
        let (mut request, buf_reader) = Request::req_from_buf(&mut stream);
        // println!("{:?}", &request);
        match request.path.as_str() {
            "/" => {
                stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n").unwrap();
                stream.flush()?;
            }
            path if path.starts_with("/echo") => {
                let echo_path = path.strip_prefix("/echo/").unwrap().to_string();
                let response = HttpTemplate::PlainText.format(&echo_path);
                stream.write_all(response.as_bytes())?;
                stream.flush()?;
            }
            path if path.starts_with("/user-agent") => {
                // dbg!(&request);
                let string = String::new();
                let user_agent = request.headers.get("user-agent").unwrap_or(&string);
                let response = HttpTemplate::PlainText.format(user_agent);
                stream.write_all(response.as_bytes())?;
                stream.flush()?;
            }
            path if path.starts_with("/files/")
                && DIR.as_ref().is_some()
                && &request.method == "GET" =>
            {
                let file_path = path.strip_prefix("/files/").unwrap();
                let full_path = std::path::Path::new(DIR.as_ref().unwrap()).join(file_path);
                match fs::read_to_string(full_path) {
                    Ok(file) => {
                        let response = HttpTemplate::OctetStream.format(&file);
                        stream.write_all(response.as_bytes())?;
                    }
                    Err(e) => {
                        eprintln!("File doesnt exists: {e}");
                        stream.write_all(HttpTemplate::not_found().as_bytes())?;
                    }
                };
                stream.flush()?;
            }
            path if path.starts_with("/files/")
                && DIR.as_ref().is_some()
                && &request.method == "POST" =>
            {
                let file_path = path.strip_prefix("/files/").unwrap();
                let full_path = std::path::Path::new(DIR.as_ref().unwrap()).join(file_path);
                let content_length = request
                    .headers
                    .get("content-length")
                    .unwrap()
                    .parse::<usize>()
                    .unwrap();
                Body::read_body(&mut request.body, content_length, buf_reader);
                fs::write(full_path, request.body.unwrap().as_bytes()).unwrap();
                stream.write_all(
                    HttpTemplate::Created
                        .format(&file_path.to_string())
                        .as_bytes(),
                )?;
                stream.flush()?;
            }
            _ => {
                stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n")?;
                stream.flush()?;
            }
        }
        if let Some(connection) = request.headers.get("connection") {
            if connection == "close" {
                break;
            }
        }
        stream.flush()?;
    }
    Ok(())
}

// pub fn handling_route(uri: String) {todo!()}
