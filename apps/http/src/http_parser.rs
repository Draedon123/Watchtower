use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt};

use crate::{body::Body, header::Headers};

/// Assumes the Start Line has been read already
pub async fn parse_http_headers_and_content<R>(
  buffer_reader: &mut R,
) -> Result<(Headers, Body), String>
where
  R: AsyncBufRead + Unpin,
{
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
    // TODO don't error out when connection closed
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
    .map(|length| length.parse::<usize>().unwrap_or(0));

  let transfer_encoding = headers.get_header("Transfer-Encoding");

  let body = match (content_length, transfer_encoding) {
    (Some(content_length),_) if content_length > 0 => {
      read_unchunked_body(buffer_reader, content_length).await?
    }
    (Some(_), _) /*if content_length == 0 */ => Body::None,
    (None, Some(encoding)) if encoding == "chunked" => read_chunked_body(buffer_reader).await?,
    (None, Some(_)) /* if encoding != "chunked" */ => todo!(),
    (None, None) => Body::None,
  };

  Ok((headers, body))
}

async fn read_unchunked_body<R>(
  buffer_reader: &mut R,
  body_length: usize,
) -> Result<Body, String>
where
  R: AsyncBufRead + Unpin,
{
  let mut body = vec![0u8; body_length];
  buffer_reader.read_exact(&mut body).await.map_err(|error| {
    format!("Failed to read {body_length} bytes from buffer: {error}")
  })?;

  Ok(Body::Binary(body))
}

async fn read_chunked_body<R>(buffer_reader: &mut R) -> Result<Body, String>
where
  R: AsyncBufRead + Unpin,
{
  let mut body = Vec::<u8>::new();
  loop {
    let mut line = String::new();

    let bytes_read = buffer_reader
      .read_line(&mut line)
      .await
      .map_err(|error| format!("Could not read line: {error}"))?;

    // reached EOF
    // TODO don't error out when connection closed
    if bytes_read == 0 {
      return Err(String::from("Connection closed by peer"));
    }

    if line.is_empty() {
      break;
    }

    let chunk_size = line
      .strip_suffix("\r\n")
      .and_then(|size| usize::from_str_radix(size, 16).ok())
      .ok_or(format!("Malformed content length: {line}"))?;

    // end of body
    if chunk_size == 0 {
      break;
    }

    let current_body_size = body.len();
    body.resize(current_body_size + chunk_size, 0);

    let bytes_read = buffer_reader
      .read_exact(&mut body[current_body_size..])
      .await
      .map_err(|error| format!("Could not read chunk: {error}"))?;

    if bytes_read != chunk_size {
      return Err(format!(
        "Could only read {bytes_read} bytes from of chunk of size {chunk_size}"
      ));
    }

    // read and discard the trailing \r\n on the chunk
    let mut dump = [0u8, 0u8];
    buffer_reader.read_exact(&mut dump).await.map_err(|error| {
      format!("Could not read chunk's trailing line-endings: {error}")
    })?;
  }

  Ok(Body::from(body))
}

#[cfg(test)]
mod tests {
  use tokio::io::BufReader;

  use super::*;

  #[tokio::test]
  async fn read_chunked() {
    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Transfer-Encoding
    let body = b"7\r\n\
    Welcome\r\n\
    1c\r\n\
    to Mozilla Developer Network\r\n\
    0\r\n\
    \r\n" as &[u8];

    let mut buffer_reader = BufReader::new(body);

    assert_eq!(
      read_chunked_body(&mut buffer_reader)
        .await
        .unwrap()
        .utf8()
        .unwrap(),
      "Welcometo Mozilla Developer Network"
    );
  }
}
