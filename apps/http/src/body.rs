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
      Self::Fixed(content) => stream
        .write_all(content.as_bytes())
        .await
        .map_err(|error| error.to_string()),

      Self::Binary(content) => stream
        .write_all(content)
        .await
        .map_err(|error| error.to_string()),
      Self::None => Ok(()),
    }
  }

  pub fn set_headers(&self, headers: &mut Headers) {
    match self {
      Self::Fixed(content) => {
        headers
          .set_header("Content-Length", &content.len().to_string())
          .set_header("Content-Type", "text/plain; charset=utf-8");
      }
      Self::Binary(content) => {
        headers
          .set_header("Content-Length", &content.len().to_string())
          .set_header("Content-Type", "application/octet-stream;");
      }
      Self::None => (),
    };
  }

  pub fn utf8(&self) -> Result<&str, String> {
    match self {
      Self::Fixed(content) => Ok(&content[..]),
      Self::Binary(content) => std::str::from_utf8(content).map_err(|error| {
        format!("Failed to convert binary data to utf8: {error}")
      }),
      Self::None => Ok(""),
    }
  }
}

impl From<String> for Body {
  fn from(value: String) -> Self {
    Self::Fixed(value)
  }
}

impl From<&str> for Body {
  fn from(value: &str) -> Self {
    Self::Fixed(value.to_string())
  }
}
