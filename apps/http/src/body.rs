use tokio::{io::AsyncWriteExt, net::TcpStream};

pub struct Body {
  pub content: String,
}

impl Body {
  pub fn new() -> Body {
    Body {
      content: String::new(),
    }
  }

  pub async fn write(&self, stream: &mut TcpStream) -> Result<(), String> {
    if let Err(error) = stream.write_all(self.content.as_bytes()).await {
      Err(error.to_string())
    } else {
      Ok(())
    }
  }
}
