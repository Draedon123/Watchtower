use std::io::BufReader;
use std::{io::prelude::*, net::TcpStream};

use crate::method::HTTPMethod;
use crate::{header::Headers, response::Response};

pub struct Request {
  pub method: HTTPMethod,
  pub path: String,
  pub headers: Headers,

  stream: TcpStream,
  content: String,
}

impl Request {
  pub fn new(stream: &TcpStream) -> Request {
    Request {
      method: HTTPMethod::Get(),
      path: String::new(),
      content: String::new(),
      headers: Headers::new(),
      stream: stream.try_clone().unwrap(),
    }
  }

  pub fn set_content(&mut self, content: &str) {
    self.content = content.to_string();

    self
      .headers
      .set_header("Content-Length", &self.content.len().to_string())
      .set_header("Content-Type", "text/plain; charset=utf-8");
  }

  pub fn get_content(&self) -> &str {
    &self.content
  }

  pub fn respond(&mut self, response: &Response) -> Result<(), String> {
    if let Ok(_) = self.stream.write_all(response.to_string().as_bytes()) {
      Ok(())
    } else {
      Err(String::from("Failed to write content to TcpStream"))
    }
  }

  pub fn send(&mut self) -> Result<(), String> {
    let content = format!(
      "{} {} HTTP/1.1\r\n{}\r\n{}",
      self.method, self.path, self.headers, self.content
    );

    if let Err(error) = self.stream.write_all(content.as_bytes()) {
      return Err(format!("Failed to write content to TcpStream: {error}"));
    };

    let mut response = [0; 4096];
    match self.stream.read(&mut response) {
      Ok(_) => Ok(()),
      Err(_) => Err(String::from("Failed to read response")),
    }
  }

  pub fn read_from_buffer<R: BufRead>(
    stream: &TcpStream,
    reader: &mut R,
  ) -> Result<Option<Request>, String> {
    let mut buffer_reader = BufReader::new(reader);
    let mut lines = Vec::<String>::new();

    let mut headers = Headers::new();

    let mut i = 1;
    loop {
      let mut line = String::new();

      if let Err(_) = buffer_reader.read_line(&mut line) {
        return Err(format!("Could not read line {i}"));
      };

      if line == "\r\n" || line == "\n" || line.is_empty() {
        break;
      }

      if i != 1 {
        if let Err(_) = headers.add_header_line(&line) {
          eprintln!("Malformed header received: {line}");
        };
      };

      lines.push(line.trim().to_string());
      i += 1;
    }

    if lines.is_empty() {
      return Ok(None);
    };

    let start_line: Vec<_> = lines[0].split(" ").collect();

    let method = HTTPMethod::from(start_line[0]);
    let path = start_line[1].to_string();

    let content_length = headers
      .get_header("Content-Length")
      .unwrap_or(&String::from("0"))
      .parse::<usize>()
      .unwrap_or(0);

    let mut request = Request {
      method,
      path,
      headers,
      stream: stream.try_clone().unwrap(),
      content: String::new(),
    };

    if content_length > 0 {
      let mut content = vec![0; content_length];

      if let Err(_) = buffer_reader.read_exact(&mut content) {
        return Err(String::from("Failed to read content from request"));
      };

      request.content = if let Ok(content) = String::from_utf8(content) {
        content
      } else {
        return Err(String::from("Failed to read content from request"));
      };
    }

    Ok(Some(request))
  }
}
