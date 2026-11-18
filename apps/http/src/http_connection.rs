use tokio::{
  io::{AsyncWriteExt, BufReader, BufWriter},
  net::{
    TcpStream,
    tcp::{OwnedReadHalf, OwnedWriteHalf},
  },
};

use crate::{request::Request, response::Response};

pub struct HttpConnection {
  reader: BufReader<OwnedReadHalf>,
  writer: BufWriter<OwnedWriteHalf>,
}

impl HttpConnection {
  pub fn new(stream: TcpStream) -> Self {
    let (read, write) = stream.into_split();
    let reader = BufReader::new(read);
    let writer = BufWriter::new(write);

    Self { reader, writer }
  }

  pub async fn send_request(
    &mut self,
    request: &mut Request,
  ) -> Result<Response, String> {
    self.send_request_rude(request).await?;
    Response::read_from_buffer(&mut self.reader).await
  }

  /// Does not wait for or read response
  pub async fn send_request_rude(
    &mut self,
    request: &mut Request,
  ) -> Result<(), String> {
    request.body.set_headers(&mut request.headers);

    self
      .writer
      .write_all(request.serialise_header().as_bytes())
      .await
      .map_err(|error| format!("Failed to write content to buffer: {error}"))?;

    request
      .body
      .write(&mut self.writer)
      .await
      .map_err(|error| format!("Failed to write content to buffer: {error}"))?;

    self
      .writer
      .flush()
      .await
      .map_err(|error| format!("Failed to flush buffer: {error}"))?;

    Ok(())
  }

  pub async fn send_response(
    &mut self,
    response: &mut Response,
  ) -> Result<(), String> {
    response.body.set_headers(&mut response.headers);

    self
      .writer
      .write_all(response.serialise_header().as_bytes())
      .await
      .map_err(|error| format!("Failed to write header to buffer: {error}"))?;

    response
      .body
      .write(&mut self.writer)
      .await
      .map_err(|error| format!("Failed to write body to buffer: {error}"))?;

    self
      .writer
      .flush()
      .await
      .map_err(|error| format!("Failed to flush buffer: {error}"))?;

    Ok(())
  }

  pub fn into_stream(self) -> TcpStream {
    self
      .reader
      .into_inner()
      .reunite(self.writer.into_inner())
      // .reunite should only fail if the two halves were from different sockets, which should be
      // impossible, given the private .reader and .writer fields
      .unwrap()
  }

  pub fn get_reader(&mut self) -> &mut BufReader<OwnedReadHalf> {
    &mut self.reader
  }

  pub fn get_writer(&mut self) -> &mut BufWriter<OwnedWriteHalf> {
    &mut self.writer
  }
}
