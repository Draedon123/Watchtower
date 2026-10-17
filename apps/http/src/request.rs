use std::{
  io::{BufReader, prelude::*},
  net::TcpStream,
};

use crate::method::HTTPMethod;
use crate::{header::Headers, response::Response};

pub struct Request {
  pub method: HTTPMethod,
  pub path: String,
  pub content: Option<String>,
  pub headers: Headers,

  stream: Option<TcpStream>,
}

impl Request {
  pub fn new(stream: TcpStream) -> Request {
    let mut request = Request::empty();
    let mut buffer_reader = BufReader::new(&stream);
    let mut lines = Vec::<String>::new();

    let mut i = 1;
    loop {
      let mut line = String::new();

      buffer_reader.read_line(&mut line).unwrap();

      if line == "\r\n" || line == "\n" || line.is_empty() {
        break;
      }

      if i != 1 {
        request.headers.add_header_line(&line);
      };

      lines.push(line.trim().to_string());
      i += 1;
    }

    let start_line: Vec<_> = lines[0].split(" ").collect();

    request.method = HTTPMethod::from(start_line[0]);
    request.path = start_line[1].to_string();

    let content_length = request
      .headers
      .get_header("Content-Length")
      .unwrap_or(&String::from("0"))
      .parse::<usize>()
      .unwrap_or(0);

    if content_length > 0 {
      let mut content = vec![0; content_length];

      buffer_reader.read_exact(&mut content).unwrap();

      request.content = Some(String::from_utf8(content).unwrap());
    }

    request.stream = Some(stream);
    request
  }

  fn empty() -> Request {
    Request {
      method: HTTPMethod::Get(),
      path: String::new(),
      content: None,
      headers: Headers::new(),
      stream: None,
    }
  }

  pub fn respond(&mut self, response: &Response) {
    self
      .stream
      .as_mut()
      .unwrap()
      .write_all(response.to_string().as_bytes())
      .unwrap();
  }
}
