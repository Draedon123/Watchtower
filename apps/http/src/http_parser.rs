use tokio::{
  io::{AsyncBufReadExt, AsyncReadExt, BufReader},
  net::TcpStream,
};

use crate::{body::Body, header::Headers};

/// Assumes the Start Line has been read already
pub async fn parse_http_headers_and_content(
  buffer_reader: &mut BufReader<TcpStream>,
) -> Result<(Headers, Body), String> {
  let mut headers = Headers::new();
  let mut line_number = 2;

  loop {
    let mut line = String::new();

    let bytes_read = buffer_reader
      .read_line(&mut line)
      .await
      .map_err(|error| format!("Could not read line {line_number}: {error}"))?;
    line_number += 1;

    // reached EOF
    if bytes_read == 0 {
      return Err(String::from("Connection closed by peer"));
    }

    if line == "\r\n" || line == "\n" || line.is_empty() {
      break;
    }

    if let Err(error) = headers.add_header_line(&line) {
      eprintln!("{error}");
    };
  }

  // TODO: account for chunked transfer

  let content_length = headers
    .get_header("Content-Length")
    .map(|length| length.parse::<usize>().unwrap_or(0))
    .unwrap_or(0);

  if content_length == 0 {
    let body = Body::None;

    return Ok((headers, body));
  }

  let mut body = vec![0u8; content_length];
  buffer_reader.read_exact(&mut body).await.map_err(|error| {
    format!("Failed to read {content_length} bytes from buffer: {error}")
  })?;

  let body = Body::Binary(body);

  Ok((headers, body))
}
