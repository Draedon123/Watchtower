use crypto::hash::Sha1;
use http::header::Headers;
const WEBSOCKET_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

pub fn get_accept_key(key: &str) -> String {
  let mut concatenation =
    String::with_capacity(key.len() + WEBSOCKET_GUID.len());

  concatenation.push_str(key);
  concatenation.push_str(WEBSOCKET_GUID);

  Sha1::new(concatenation.as_bytes()).to_base64()
}

pub fn header_to_vec(headers: &Headers, header: &str) -> Vec<String> {
  headers
    .get_header(header)
    .map(|headers| {
      headers
        .split(',')
        .map(|value| value.trim().to_owned())
        .collect()
    })
    .unwrap_or_default()
}
