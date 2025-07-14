use crate::lib::{HttpTemplates, Request, DIR};
use std::{
    fs, io::{BufReader, Error, Write}, net::{TcpListener, TcpStream}, thread
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
                let stream = BufReader::new(stream);
                thread::spawn( move || handle_connection(stream));
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
    Ok(())
}

fn handle_connection(mut stream: BufReader<TcpStream>) -> Result<(), Error> {
    loop {
        let mut req = Request::from_stream(&mut stream)?;
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
                let result = req.headers.get("user-agent").unwrap().trim_end();
                HttpTemplates::PlainText.format(result)
            }
            ("GET", "files") => {
                let filename = req.segment(2).unwrap();
                let full_path = std::path::Path::new(DIR.as_ref().unwrap()).join(filename);
                dbg!(&req);
                match fs::read_to_string(full_path) {
                    Ok(file) => {
                        HttpTemplates::OctetStream.format(&file)
                    }
                    Err(e) => {
                        eprintln!("File doesnt exists: {e}");
                        HttpTemplates::NotFound.format("")
                    }
                }
            }
            _ => HttpTemplates::NotFound.format(""),
        };
        let stream = stream.get_mut();
        stream.write_all(f.as_slice())?;
        stream.flush()?;
    }
    Ok(())
}