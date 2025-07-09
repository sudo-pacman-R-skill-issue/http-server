use std::net::TcpStream;

#[derive(Debug)]
pub struct Request<'a> {
    pub method: &'a str,
    pub path: &'a str,
    pub protocol: &'a str,
    pub headers: &'a str,
    pub body: &'a str,
}

pub struct Response<'a> {
    pub method: &'a str,
    pub path: &'a str,
    pub protocol: &'a str,
    pub headers: &'a str,
    pub body: &'a str,
}

pub fn req_handler(stream: &mut TcpStream) -> Response {
    todo!()
}
