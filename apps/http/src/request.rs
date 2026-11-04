use crate::method::HTTPMethod;
use crate::url::Url;
use crate::{header::Headers, response::Response};

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub struct Request<'a> {
  pub method: HTTPMethod,
  pub path: Url,
  pub headers: Headers,
  pub stream: &'a mut TcpStream,

  content: String,
}

impl<'a> Request<'a> {
  pub fn new(stream: &'a mut TcpStream) -> Request<'a> {
    Request {
      method: HTTPMethod::Get(),
      path: Url::new(""),
      content: String::new(),
      headers: Headers::new(),
      stream,
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

  pub async fn respond(&mut self, response: &Response) -> Result<(), String> {
    if let Ok(_) = self.stream.write_all(response.to_string().as_bytes()).await {
      Ok(())
    } else {
      Err(String::from("Failed to write content to TcpStream"))
    }
  }

  pub async fn send(&mut self) -> Result<(), String> {
    let content = format!(
      "{} {} HTTP/1.1\r\n{}\r\n{}",
      self.method,
      self.path.percent_encode().unwrap(),
      self.headers,
      self.content
    );

    if let Err(error) = self.stream.write_all(content.as_bytes()).await {
      return Err(format!("Failed to write content to TcpStream: {error}"));
    };

    let mut response = [0; 4096];
    match self.stream.read(&mut response).await {
      Ok(_) => Ok(()),
      Err(_) => Err(String::from("Failed to read response")),
    }
  }

  pub async fn read_from_buffer(stream: &'a mut TcpStream) -> Result<Option<Request<'a>>, String> {
    let mut buffer_reader = BufReader::new(&mut *stream);
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
      .unwrap_or(&String::from("0"))
      .parse::<usize>()
      .unwrap_or(0);

    let mut content = vec![0; content_length];
    let content = if content_length > 0 {
      if let Err(_) = buffer_reader.read_exact(&mut content).await {
        return Err(String::from("Failed to read content from request"));
      };

      if let Ok(content) = String::from_utf8(content) {
        content
      } else {
        return Err(String::from("Failed to read content from request"));
      }
    } else {
      String::new()
    };

    let request = Request {
      method,
      path,
      headers,
      stream,
      content,
    };

    Ok(Some(request))
  }
}
