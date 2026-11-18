use tokio::{
  io::{AsyncReadExt, AsyncWriteExt, BufReader},
  net::TcpStream,
};

use crate::{request::Request, response::Response};

pub struct HttpConnection {
  pub stream: BufReader<TcpStream>,
}

impl HttpConnection {
  pub fn new(stream: TcpStream) -> Self {
    let buffer_reader = BufReader::new(stream);

    Self {
      stream: buffer_reader,
    }
  }

  pub async fn send_request(
    &mut self,
    request: &mut Request,
  ) -> Result<(), String> {
    request.body.set_headers(&mut request.headers);

    self
      .stream
      .write_all(request.serialise_header().as_bytes())
      .await
      .map_err(|error| {
        format!("Failed to write content to TcpStream: {error}")
      })?;

    request
      .body
      .write(self.stream.get_mut())
      .await
      .map_err(|error| {
        format!("Failed to write content to TcpStream: {error}")
      })?;

    // TODO: read in chunks into a Vec
    let mut response = [0; 4096];
    self
      .stream
      .read(&mut response)
      .await
      .map_err(|error| format!("Failed to read response: {error}"))?;

    Ok(())
  }

  pub async fn send_response(
    &mut self,
    response: &mut Response,
  ) -> Result<(), String> {
    response.body.set_headers(&mut response.headers);

    self
      .stream
      .write_all(response.serialise_header().as_bytes())
      .await
      .map_err(|error| {
        format!("Failed to write header to TcpStream: {error}")
      })?;

    response
      .body
      .write(self.stream.get_mut())
      .await
      .map_err(|error| format!("Failed to write body to TcpStream: {error}"))?;

    Ok(())
  }
}
