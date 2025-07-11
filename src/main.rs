use std::{
    convert::Infallible, io::{BufRead, BufReader, Error, ErrorKind, Read, Write}, net::{TcpListener, TcpStream}, ops::Index
};
use bumpalo::Bump;

use crate::lib::{HttpTemplates, Request};
mod lib;
// use crate::lib::req_handler;
mod second {
    include!("../src_1/main.rs");
}

fn main() -> Result<(), Error> {
    let listener = TcpListener::bind("127.0.0.1:4221")?;
    let bytes_buffer: Vec<u8> = Vec::with_capacity(512);
    let bump = Bump::new();
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                // dbg!(&stream);
                let req = Request::from_stream(&stream, &bump)?;
                let f = match req.path {
                    "/" => HttpTemplates::Slash.format(""),
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


