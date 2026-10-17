use crate::{header::Headers, status_codes::StatusCode};

pub struct Response {
  pub headers: Headers,
  pub status_code: StatusCode,
  pub content: String,
}

impl Response {
  pub fn new() -> Self {
    Response {
      headers: Headers::new(),
      status_code: StatusCode::Unknown(0),
      content: String::new(),
    }
  }

  pub fn set_content(&mut self, content: String) {
    self.content = content;

    self
      .headers
      .set_header("Content-Length", &self.content.len().to_string());
    self
      .headers
      .set_header("Content-Type", "text/plain; charset=utf-8");
  }
}

impl ToString for Response {
  fn to_string(&self) -> String {
    format!(
      "HTTP/1.1 {} {}\r\n{}\r\n{}",
      self.status_code.get_code(),
      self.status_code.get_message(),
      self.headers,
      self.content
    )
  }
}
