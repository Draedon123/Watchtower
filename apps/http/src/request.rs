use crate::body::Body;
use crate::header::Headers;
use crate::http_method::HttpMethod;
use crate::http_parser::parse_http_headers_and_content;
use crate::url::Url;

use tokio::io::{AsyncBufRead, AsyncBufReadExt};

pub struct Request {
  pub method: HttpMethod,
  pub path: Url,
  pub headers: Headers,
  pub body: Body,
}

impl Request {
  pub fn new() -> Self {
    Self {
      method: HttpMethod::Get(),
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
      self.headers.serialise()
    )
  }

  pub async fn read_from_buffer<R>(buffer: &mut R) -> Result<Self, String>
  where
    R: AsyncBufRead + Unpin,
  {
    let mut start_line = String::new();

    buffer
      .read_line(&mut start_line)
      .await
      .map_err(|error| format!("Failed to read start line: {error}"))?;

    let (headers, body) = parse_http_headers_and_content(buffer).await?;
    let (method, path) = Self::parse_start_line(&start_line)?;

    let request = Self {
      method,
      path,
      headers,
      body,
    };

    Ok(request)
  }

  fn parse_start_line(line: &str) -> Result<(HttpMethod, Url), String> {
    let mut split = line.split(" ");
    let method = split
      .next()
      .ok_or(format!("Malformed start line: \"{line}\""))
      .map(HttpMethod::from)?;
    let path = split
      .next()
      .ok_or(format!("Malformed start line: \"{line}\""))
      .map(Url::new)?;

    Ok((method, path))
  }
}

impl Default for Request {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn start_lines() {
    assert_eq!(
      Request::parse_start_line("POST /users HTTP/1.1"),
      Ok((HttpMethod::Post(), Url::new("/users")))
    );
  }
}
