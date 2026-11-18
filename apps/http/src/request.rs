use crate::body::Body;
use crate::header::Headers;
use crate::http_parser::parse_http_headers_and_content;
use crate::method::HTTPMethod;
use crate::url::Url;

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpStream;

pub struct Request {
  pub method: HTTPMethod,
  pub path: Url,
  pub headers: Headers,
  pub body: Body,
}

impl Request {
  pub fn new() -> Self {
    Request {
      method: HTTPMethod::Get(),
      path: Url::new(""),
      headers: Headers::new(),
      body: Body::None,
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

  pub async fn read_from_buffer(buffer: &mut BufReader<TcpStream>) -> Result<Request, String> {
    let mut start_line = String::new();

    buffer
      .read_line(&mut start_line)
      .await
      .map_err(|error| format!("Failed to read start-line: {error}"))?;

    let (headers, body) = parse_http_headers_and_content(buffer).await?;
    let (method, path) = Request::parse_start_line(&start_line)?;

    let request = Request {
      method,
      path,
      headers,
      body,
    };

    Ok(request)
  }

  fn parse_start_line(line: &str) -> Result<(HTTPMethod, Url), String> {
    let mut split = line.split(" ");
    let method = split
      .next()
      .ok_or(format!("Malformed start line: \"{line}\""))
      .map(|method| HTTPMethod::from(method))?;
    let path = split
      .next()
      .ok_or(format!("Malformed start line: \"{line}\""))
      .map(Url::new)?;

    Ok((method, path))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn start_lines() {
    assert_eq!(
      Request::parse_start_line("POST /users HTTP/1.1"),
      Ok((HTTPMethod::Post(), Url::new("/users")))
    );
  }
}
