use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::header::Headers;

pub enum Body {
  Fixed(String),
  Binary(Vec<u8>),
  None,
}

impl Body {
  pub async fn write(&self, stream: &mut TcpStream) -> Result<(), String> {
    match self {
      Body::Fixed(content) => stream
        .write_all(content.as_bytes())
        .await
        .map_err(|error| error.to_string()),

      Body::Binary(content) => stream
        .write_all(content)
        .await
        .map_err(|error| error.to_string()),
      Body::None => Ok(()),
    }
  }

  pub fn set_headers(&self, headers: &mut Headers) {
    match self {
      Body::Fixed(content) => {
        headers
          .set_header("Content-Length", &content.len().to_string())
          .set_header("Content-Type", "text/plain; charset=utf-8");
      }
      Body::Binary(content) => {
        headers
          .set_header("Content-Length", &content.len().to_string())
          .set_header("Content-Type", "application/octet-stream;");
      }
      Body::None => (),
    };
  }

  pub fn utf8(&self) -> &str {
    match self {
      Body::Fixed(content) => &content[..],
      Body::Binary(content) => std::str::from_utf8(content).unwrap(),
      Body::None => "",
    }
  }
}

impl From<String> for Body {
  fn from(value: String) -> Self {
    Body::Fixed(value)
  }
}
