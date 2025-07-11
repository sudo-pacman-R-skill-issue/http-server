use std::{
    convert::Infallible, io::{BufRead, BufReader, Error, ErrorKind, Read, Write}, net::{TcpListener, TcpStream}, ops::Index
};
use bumpalo::Bump;

use crate::lib::{HttpTemplates, Request};
mod lib;
mod second {
    include!("../src_1/main.rs");
}

fn main() -> Result<(), Error> {
    let listener = TcpListener::bind("127.0.0.1:4221")?;
    let bump = Bump::new();
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let req = Request::from_stream(&stream, &bump)?;
                let f = match (req.method, req.segment(1).unwrap_or("_")) {
                    ("GET", "echo") => {
                        let result = req.segment(2).unwrap();
                        dbg!(&result);
                        HttpTemplates::PlainText.format(result)
                    },
                    ("GET", "") => HttpTemplates::Slash.format(""),
                    ("GET", "user-agent") => {
                        let result = req.headers.get("User-Agent").unwrap().trim_end();
                        HttpTemplates::PlainText.format(result)},
                    _ => HttpTemplates::NotFound.format("")
                };
                stream.write_all(f.as_slice())?;
                stream.flush()?;
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
    Ok(())
}


