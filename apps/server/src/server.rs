use crate::config::Config;
use http::{request::Request, response::Response};
use std::net::TcpListener;
pub struct Server {}

impl Server {
  pub fn new(config: &Config) -> Result<Server, Box<dyn std::error::Error>> {
    let server_address = format!("127.0.0.1:{}", config.port);

    let listener = TcpListener::bind(&server_address)?;

    println!("Opened server on {server_address}");

    for stream in listener.incoming() {
      let stream = stream.unwrap();

      let mut request = Request::new(stream);
      let mut response = Response::new();

      response.set_content(String::from("Hello, World!"));
      response.status_code = http::status_codes::StatusCode::Status200;

      request.respond(&response);
    }

    Ok(Server {})
  }
}
