use tokio::{
  io::{AsyncReadExt, AsyncWriteExt},
  net::TcpStream,
};

use crate::{request::Request, response::Response};

pub struct HTTPConnection {
  pub stream: TcpStream,
}

impl HTTPConnection {
  pub fn new(stream: TcpStream) -> HTTPConnection {
    HTTPConnection { stream }
  }

  pub async fn send_request(&mut self, request: &mut Request) -> Result<(), String> {
    request.body.set_headers(&mut request.headers);

    if let Err(error) = self
      .stream
      .write_all(request.serialise_header().as_bytes())
      .await
    {
      return Err(format!("Failed to write content to TcpStream: {error}"));
    };

    if let Err(error) = request.body.write(&mut self.stream).await {
      return Err(format!("Failed to write content to TcpStream: {error}"));
    };

    let mut response = [0; 4096];
    match self.stream.read(&mut response).await {
      Ok(_) => Ok(()),
      Err(_) => Err(String::from("Failed to read response")),
    }
  }

  pub async fn send_response(&mut self, response: &mut Response) -> Result<(), String> {
    response.body.set_headers(&mut response.headers);

    if let Err(_) = self
      .stream
      .write_all(response.serialise_header().as_bytes())
      .await
    {
      return Err(String::from("Failed to write content to TcpStream"));
    };

    if let Err(_) = response.body.write(&mut self.stream).await {
      return Err(String::from("Failed to write content to TcpStream"));
    };

    Ok(())
  }
}
