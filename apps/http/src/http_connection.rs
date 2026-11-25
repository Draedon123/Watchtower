use tokio::{
  io::{AsyncBufRead, AsyncWrite, AsyncWriteExt, BufReader, BufWriter},
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
    send_request(&mut self.reader, &mut self.writer, request).await
  }

  /// Does not wait for or read response
  pub async fn send_request_rude(
    &mut self,
    request: &mut Request,
  ) -> Result<(), String> {
    send_request_rude(&mut self.writer, request).await
  }

  pub async fn send_response(
    &mut self,
    response: &mut Response,
  ) -> Result<(), String> {
    send_response(&mut self.writer, response).await
  }

  pub fn into_stream(self) -> TcpStream {
    self
      .reader
      .into_inner()
      .reunite(self.writer.into_inner())
      // .reunite should only fail if the two halves were from different sockets, which should be
      // impossible, given the private .reader and .writer fields
      .expect("Read and write halves should be from same TcpStream")
  }

  pub fn get_reader(&mut self) -> &mut BufReader<OwnedReadHalf> {
    &mut self.reader
  }

  pub fn get_writer(&mut self) -> &mut BufWriter<OwnedWriteHalf> {
    &mut self.writer
  }
}

pub async fn send_request<R, W>(
  reader: &mut R,
  writer: &mut W,
  request: &mut Request,
) -> Result<Response, String>
where
  R: AsyncBufRead + Unpin,
  W: AsyncWrite + Unpin,
{
  send_request_rude(writer, request).await?;
  Response::read_from_buffer(reader).await
}

/// Does not wait for or read response
pub async fn send_request_rude<W>(
  writer: &mut W,
  request: &mut Request,
) -> Result<(), String>
where
  W: AsyncWrite + Unpin,
{
  request.body.set_headers(&mut request.headers);

  writer
    .write_all(request.serialise_header().as_bytes())
    .await
    .map_err(|error| format!("Failed to write content to buffer: {error}"))?;

  request
    .body
    .write(writer)
    .await
    .map_err(|error| format!("Failed to write content to buffer: {error}"))?;

  writer
    .flush()
    .await
    .map_err(|error| format!("Failed to flush buffer: {error}"))?;

  Ok(())
}

pub async fn send_response<W>(
  writer: &mut W,
  response: &mut Response,
) -> Result<(), String>
where
  W: AsyncWrite + Unpin,
{
  response.body.set_headers(&mut response.headers);

  writer
    .write_all(response.serialise_header().as_bytes())
    .await
    .map_err(|error| format!("Failed to write header to buffer: {error}"))?;

  response
    .body
    .write(writer)
    .await
    .map_err(|error| format!("Failed to write body to buffer: {error}"))?;

  writer
    .flush()
    .await
    .map_err(|error| format!("Failed to flush buffer: {error}"))?;

  Ok(())
}
