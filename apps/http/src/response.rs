use crate::{body::Body, header::Headers, status_codes::StatusCode};

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
}
