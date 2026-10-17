use std::{
  io::{BufReader, prelude::*},
  net::TcpStream,
};

use crate::http::method::HTTPMethod;

pub struct Request {
  method: HTTPMethod,
  path: String,
  content: Option<String>,
}

impl Request {
  pub fn new(stream: &TcpStream) -> Request {
    let mut buffer_reader = BufReader::new(stream);
    let mut lines = Vec::<String>::new();

    loop {
      let mut line = String::new();

      buffer_reader.read_line(&mut line).unwrap();

      if line == "\r\n" || line == "\n" || line.is_empty() {
        break;
      }

      lines.push(line.trim().to_string());
    }

    // println!("{:#?}", lines);

    let mut request = Request::empty();

    let start_line: Vec<_> = lines[0].split(" ").collect();

    request.method = HTTPMethod::from(start_line[0]);
    request.path = start_line[1].to_string();

    let content_length = match lines.iter().find(|line| line.starts_with("Content-Length")) {
      Some(header) => header.split(":").collect::<Vec<_>>()[1]
        .trim()
        .parse::<usize>()
        .unwrap(),
      None => 0,
    };

    if content_length > 0 {
      let mut content = vec![0; content_length];

      buffer_reader.read_exact(&mut content).unwrap();

      request.content = Some(String::from_utf8(content).unwrap());
    }

    println!("Received {} request for {}", request.method, request.path);
    println!("Body is {} bytes: {:#?}", content_length, request.content);

    request
  }

  fn empty() -> Request {
    Request {
      method: HTTPMethod::Get(),
      path: String::new(),
      content: None,
    }
  }
}
