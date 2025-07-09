mod lib;
use std::net::TcpListener;

use crate::lib::*;

mod second {
    include!("../src_1/main.rs");
}
fn main() {
    let req = Request {
        method: "test",
        path: "test",
        protocol: "test",
        headers: "test",
        body: "test",
    };
    println!("{:?}", std::mem::size_of::<Request>());
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                println!("accepted new connection");
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }

}