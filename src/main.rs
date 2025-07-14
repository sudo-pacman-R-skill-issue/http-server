use crate::lib::{HttpTemplates, Request};
use std::{
    io::{Error, ErrorKind, Write},
    net::{TcpListener, TcpStream}, thread,
};
mod lib;
mod second {
    include!("../src_1/main.rs");
}

fn main() -> Result<(), Error> {
    let listener = TcpListener::bind("127.0.0.1:4221")?;
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn( move || handle_connection(stream));
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
    Ok(())
}

fn handle_connection(mut stream: TcpStream) -> Result<(), Error> {
    loop {
        let req = Request::from_stream(&stream)?;
        if let Some(connection) = req.headers.get("connection") {
            if connection == "close" {
                break;
            }
        }
        let f = match (req.method.as_str(), req.segment(1).unwrap_or("_")) {
            ("GET", "echo") => {
                let result = req.segment(2).unwrap();
                // dbg!(&result);
                HttpTemplates::PlainText.format(result)
            }
            ("GET", "") => HttpTemplates::Slash.format(""),
            ("GET", "user-agent") => {
                let result = req.headers.get("User-Agent").unwrap().trim_end();
                HttpTemplates::PlainText.format(result)
            }
            _ => HttpTemplates::NotFound.format(""),
        };
        stream.write_all(f.as_slice())?;
        stream.flush()?;
    }
    Ok(())
}