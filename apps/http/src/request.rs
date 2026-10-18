use std::{
  io::{BufReader, prelude::*},
  net::TcpStream,
};

use crate::method::HTTPMethod;
use crate::{header::Headers, response::Response};

pub struct Request {
  pub method: HTTPMethod,
  pub path: String,
  pub headers: Headers,

  content: String,
  stream: Option<TcpStream>,
}

impl Request {
  pub fn new(address: &str) -> Request {
    Request {
      method: HTTPMethod::Get(),
      path: String::new(),
      content: String::new(),
      headers: Headers::new(),
      stream: Some(TcpStream::connect(address).unwrap()),
    }
  }

  pub fn read_from_stream(stream: TcpStream) -> Request {
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

      request.content = String::from_utf8(content).unwrap();
    }

    request.stream = Some(stream);
    request
  }

  fn empty() -> Request {
    Request {
      method: HTTPMethod::Get(),
      path: String::new(),
      content: String::new(),
      headers: Headers::new(),
      stream: None,
    }
  }

  pub fn set_content(&mut self, content: &str) {
    self.content = content.to_string();

    self
      .headers
      .set_header("Content-Length", &self.content.len().to_string());
    self
      .headers
      .set_header("Content-Type", "text/plain; charset=utf-8");
  }

  pub fn get_content(&self) -> &str {
    &self.content
  }

  pub fn respond(&mut self, response: &Response) {
    self
      .stream
      .as_mut()
      .unwrap()
      .write_all(response.to_string().as_bytes())
      .unwrap();
  }

  pub fn send(&mut self) {
    let content = format!(
      "{} {} HTTP/1.1\r\n{}\r\n{}",
      self.method, self.path, self.headers, self.content
    );

    println!("{}", content);

    self
      .stream
      .as_mut()
      .unwrap()
      .write_all(content.as_bytes())
      .unwrap();
  }
}
