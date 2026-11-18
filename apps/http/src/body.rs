use std::pin::Pin;

use tokio::io::{AsyncWrite, AsyncWriteExt};
use tokio_stream::{Stream, StreamExt};

use crate::header::Headers;

pub enum Body {
  Fixed(String),
  Binary(Vec<u8>),
  Chunked(Pin<Box<dyn Stream<Item = Vec<u8>> + Send + Sync>>),
  None,
}

impl Body {
  /// Does not flush buffer
  pub async fn write<W>(&mut self, stream: &mut W) -> Result<(), String>
  where
    W: AsyncWrite + Unpin,
  {
    match self {
      Self::Fixed(content) => stream
        .write_all(content.as_bytes())
        .await
        .map_err(|error| error.to_string()),

      Self::Binary(content) => stream
        .write_all(content)
        .await
        .map_err(|error| error.to_string()),
      Self::Chunked(content) => {
        while let Some(data) = content.next().await {
          (async || -> std::io::Result<()> {
            stream
              .write_all(format!("{}\r\n", data.len()).as_bytes())
              .await?;

            stream.write_all(&data).await?;
            stream.write_all(b"\r\n").await?;

            Ok(())
          })()
          .await
          .map_err(|error| error.to_string())?;
        }

        stream
          .write_all(b"0\r\n\r\n")
          .await
          .map_err(|error| error.to_string())?;

        Ok(())
      }
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
          .set_header("Content-Type", "application/octet-stream");
      }
      Self::Chunked(_) => {
        headers
          .set_header("Content-Type", "application/octet-stream")
          .set_header("Transfer-Encoding", "chunked");
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
      Self::Chunked(_) => todo!(),
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
