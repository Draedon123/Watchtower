use tokio::io::{AsyncBufRead, AsyncBufReadExt};

use crate::{
  body::Body, header::Headers, http_parser::parse_http_headers_and_content,
  http_status_codes::HttpStatusCode,
};

#[derive(Debug)]
pub struct Response {
  pub headers: Headers,
  pub status_code: HttpStatusCode,
  pub body: Body,
}

impl Response {
  pub fn new() -> Self {
    Self {
      headers: Headers::new(),
      status_code: HttpStatusCode::Unknown(0),
      body: Body::None,
    }
  }

  pub fn serialise_header(&self) -> String {
    format!(
      "HTTP/1.1 {} {}\r\n{}\r\n",
      self.status_code.get_code(),
      self.status_code.get_message(),
      self.headers.serialise(),
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
    let status_code = Self::parse_start_line(&start_line)?;

    Ok(Self {
      headers,
      body,
      status_code,
    })
  }

  fn parse_start_line(line: &str) -> Result<HttpStatusCode, String> {
    let mut split = line.split(" ");
    // HTTP Version, e.g., HTTP/1.1
    split.next();
    let status_code = split
      .next()
      .ok_or(format!("Malformed start line: \"{line}\""))?;
    let status_code = status_code
      .parse::<u16>()
      .map_err(|_| format!("Malformed start line: \"{line}\""))?;
    let status_code = HttpStatusCode::from(status_code);

    Ok(status_code)
  }
}

impl Default for Response {
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
      Response::parse_start_line("HTTP/1.1 403 Forbidden"),
      Ok(HttpStatusCode::Status403)
    );
  }
}
