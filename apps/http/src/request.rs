use crate::body::Body;
use crate::header::Headers;
use crate::method::HTTPMethod;
use crate::url::Url;

use tokio::io::AsyncReadExt;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpStream;

pub struct Request {
  pub method: HTTPMethod,
  pub path: Url,
  pub headers: Headers,
  pub body: Body,
}

impl Request {
  pub fn new() -> Request {
    Request {
      method: HTTPMethod::Get(),
      path: Url::new(""),
      headers: Headers::new(),
      body: Body::new(),
    }
  }

  pub fn serialise_header(&self) -> String {
    format!(
      "{} {} HTTP/1.1\r\n{}\r\n",
      self.method,
      self
        .path
        .percent_encode()
        .unwrap_or_else(|_| self.path.raw.clone()),
      self.headers
    )
  }

  pub async fn read_from_buffer(stream: &mut TcpStream) -> Result<Option<Request>, String> {
    let mut buffer_reader = BufReader::new(stream);
    let mut lines = Vec::<String>::new();
    let mut headers = Headers::new();

    let mut i = 1;
    loop {
      let mut line = String::new();

      if let Err(_) = buffer_reader.read_line(&mut line).await {
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
    let path = Url::new(start_line[1]);

    let content_length = headers
      .get_header("Content-Length")
      .map(|length| length.parse::<usize>().unwrap_or(0))
      .unwrap_or(0);

    let mut content = vec![0; content_length];
    let body = if content_length > 0 {
      if let Err(_) = buffer_reader.read_exact(&mut content).await {
        return Err(String::from("Failed to read content from request"));
      };

      if let Ok(content) = String::from_utf8(content) {
        Body { content }
      } else {
        return Err(String::from("Failed to read content from request"));
      }
    } else {
      Body::new()
    };

    let request = Request {
      method,
      path,
      headers,
      body,
    };

    Ok(Some(request))
  }
}
