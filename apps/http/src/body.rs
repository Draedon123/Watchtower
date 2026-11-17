use std::fmt::Display;

use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::header::Headers;

pub enum Body {
  Fixed(String),
  None,
}

impl Body {
  pub async fn write(&self, stream: &mut TcpStream) -> Result<(), String> {
    match self {
      Body::Fixed(content) => {
        if let Err(error) = stream.write_all(content.as_bytes()).await {
          Err(error.to_string())
        } else {
          Ok(())
        }
      }
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
      Body::None => (),
    };
  }
}

impl From<String> for Body {
  fn from(value: String) -> Self {
    Body::Fixed(value)
  }
}

impl Display for Body {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Body::Fixed(content) => write!(f, "{}", content),
      Body::None => Ok(()),
    }
  }
}
