use crypto::hash::Sha1;

const WEBSOCKET_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

pub fn get_accept_key(key: &str) -> String {
  let mut concatenation =
    String::with_capacity(key.len() + WEBSOCKET_GUID.len());

  concatenation.push_str(key);
  concatenation.push_str(WEBSOCKET_GUID);

  Sha1::new(concatenation.as_bytes()).to_base64()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn handshake_key() {
    // https://datatracker.ietf.org/doc/html/rfc6455#section-4.2.2

    let key = "dGhlIHNhbXBsZSBub25jZQ==";

    assert_eq!(get_accept_key(key), "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=");
  }
}
