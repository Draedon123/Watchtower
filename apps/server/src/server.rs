use crate::{config::Config, request::Request};
use std::{io::prelude::*, net::TcpListener};
pub struct Server {}

impl Server {
  pub fn new(config: &Config) -> Result<Server, Box<dyn std::error::Error>> {
    let server_address = format!("127.0.0.1:{}", config.port);

    let listener = TcpListener::bind(&server_address)?;

    println!("Opened server on {server_address}");

    for stream in listener.incoming() {
      let mut stream = stream.unwrap();

      Request::new(&stream);

      stream
        .write_all("HTTP/1.1 200 OK\r\n\r\n".as_bytes())
        .unwrap();
    }

    Ok(Server {})
  }
}
