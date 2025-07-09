#[allow(unused_imports)]
use std::convert::Infallible;
use std::error::Error;
use bytes::Bytes;
use http_body_util::Full;
use hyper::{server::conn::http1, service::service_fn, Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
/// codecrafters doesnt allow it because anti-cheat reacting on it
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").await.unwrap();
    loop {
        let (stream, _) = listener.accept().await?;
                let io = TokioIo::new(stream);
                tokio::task::spawn(async move {
                    if let Err(e) = http1::Builder::new().serve_connection(io, service_fn(dvuhsotaya)).await {
                        eprintln!("Error serving connection {e}")
                    }

                });
        }
}

async fn dvuhsotaya(_: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>,Infallible> {
    Ok(Response::new(Full::new(Bytes::from("\r\n\r\n"))))
}
