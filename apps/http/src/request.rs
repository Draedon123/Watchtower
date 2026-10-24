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
  pub stream: Option<TcpStream>,

  content: String,
}

impl Request {
  pub fn new(address: &str) -> std::io::Result<Request> {
    let stream = TcpStream::connect(address)?;

    Ok(Request {
      method: HTTPMethod::Get(),
      path: String::new(),
      content: String::new(),
      headers: Headers::new(),
      stream: Some(stream),
    })
  }

  pub fn new_auto_handle_errors(address: &str) -> Request {
    let request = Request::new(address);

    match request {
      Ok(request) => request,
      Err(error) => {
        match error.kind() {
          std::io::ErrorKind::NetworkUnreachable => {
            eprintln!("Could not reach network");
          }
          std::io::ErrorKind::HostUnreachable => {
            eprintln!("Could not reach host {}", address);
          }
          _ => {
            eprintln!("Unknown error: {:#?}", error);
          }
        };

        Request::empty()
      }
    }
  }

  pub fn read_from_stream(stream: TcpStream) -> Result<Request, String> {
    let mut request = Request::empty();
    let mut buffer_reader = BufReader::new(&stream);
    let mut lines = Vec::<String>::new();

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
        if let Err(_) = request.headers.add_header_line(&line) {
          eprintln!("Malformed header received: {line}");
        };
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

      if let Err(_) = buffer_reader.read_exact(&mut content) {
        return Err(String::from("Failed to read content from request"));
      };

      request.content = if let Ok(content) = String::from_utf8(content) {
        content
      } else {
        return Err(String::from("Failed to read content from request"));
      };
    }

    request.stream = Some(stream);
    Ok(request)
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
      .set_header("Content-Length", &self.content.len().to_string())
      .set_header("Content-Type", "text/plain; charset=utf-8");
  }

  pub fn get_content(&self) -> &str {
    &self.content
  }

  pub fn respond(&mut self, response: &Response) -> Result<(), String> {
    let stream = if let Some(stream) = self.stream.as_mut() {
      stream
    } else {
      return Err(String::from(
        "Failed to obtain mutable reference to TcpStream",
      ));
    };

    if let Ok(_) = stream.write_all(response.to_string().as_bytes()) {
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

    let stream = if let Some(stream) = self.stream.as_mut() {
      stream
    } else {
      return Err(String::from(
        "Failed to obtain mutable reference to TcpStream",
      ));
    };

    if let Err(_) = stream.write_all(content.as_bytes()) {
      return Err(String::from("Failed to write content to TcpStream"));
    };

    let mut response = [0; 4096];
    match stream.read(&mut response) {
      Ok(_) => Ok(()),
      Err(_) => Err(String::from("Failed to read response")),
    }
  }
}
