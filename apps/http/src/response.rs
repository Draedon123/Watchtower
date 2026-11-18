use tokio::{
  io::{AsyncBufReadExt, BufReader},
  net::TcpStream,
};

use crate::{
  body::Body, header::Headers, http_parser::parse_http_headers_and_content,
  status_codes::StatusCode,
};

pub struct Response {
  pub headers: Headers,
  pub status_code: StatusCode,
  pub body: Body,
}

impl Response {
  pub fn new() -> Self {
    Response {
      headers: Headers::new(),
      status_code: StatusCode::Unknown(0),
      body: Body::None,
    }
  }

  pub fn serialise_header(&self) -> String {
    format!(
      "HTTP/1.1 {} {}\r\n{}\r\n",
      self.status_code.get_code(),
      self.status_code.get_message(),
      self.headers,
    )
  }

  pub async fn read_from_buffer(buffer: &mut BufReader<TcpStream>) -> Result<Response, String> {
    let mut start_line = String::new();

    buffer
      .read_line(&mut start_line)
      .await
      .map_err(|error| format!("Failed to read start-line: {error}"))?;

    let (headers, body) = parse_http_headers_and_content(buffer).await?;
    let status_code = Response::parse_start_line(&start_line)?;

    Ok(Response {
      headers,
      body,
      status_code,
    })
  }

  fn parse_start_line(line: &str) -> Result<StatusCode, String> {
    let mut split = line.split(" ");
    // HTTP Version, e.g., HTTP/1.1
    split.next();
    let status_code = split.next().ok_or(String::from("Malformed start line"))?;
    let status_code = status_code
      .parse::<u16>()
      .map_err(|_| String::from("Malformed start line"))?;
    let status_code = StatusCode::from(status_code);

    Ok(status_code)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn start_lines() {
    assert_eq!(
      Response::parse_start_line("HTTP/1.1 403 Forbidden"),
      Ok(StatusCode::Status403)
    );
  }
}
