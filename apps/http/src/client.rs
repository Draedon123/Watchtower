use std::net::TcpStream;

use crate::{header::Headers, method::HTTPMethod, request::Request, url::Url};

pub struct Client {
  stream: Option<TcpStream>,
  host: String,
  port: u16,
}

impl Client {
  pub fn new() -> Client {
    Client {
      stream: None,
      host: String::new(),
      port: 0,
    }
  }

  pub fn request(
    &mut self,
    method: HTTPMethod,
    address: &str,
    headers: Option<Headers>,
  ) -> Request {
    let url = Url::new(address);

    if let Ok(protocol) = url.protocol() {
      if protocol != "http" {
        unimplemented!("Only HTTP requests are supported")
      }
    };

    let host = url.host().unwrap();
    let path = url.path().unwrap();
    let port = url
      .port()
      .unwrap_or_else(|_| url.guess_port().unwrap_or(80));

    if self.host != host || self.port != port {
      self.host = host.clone();
      self.port = port;

      self.stream = Some(TcpStream::connect((host, port)).unwrap());
    };

    if let Some(stream) = self.stream.as_ref() {
      let mut request = Request::new(stream);
      request.method = method;
      request.path = path;

      if let Some(headers) = headers {
        request.headers = headers;
      };

      request
    } else {
      panic!("No connection made");
    }
  }
}
