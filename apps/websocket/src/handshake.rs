use crypto::base_64;
use http::{http_method::HttpMethod, request::Request};
use misc::uri::Uri;

use crate::utils::header_to_vec;

#[derive(Debug, PartialEq, Eq)]
pub struct Handshake {
  pub resource_name: String,
  pub host: String,
  pub key: String,
  pub protocols: Vec<String>,
  pub extensions: Vec<String>,
  pub version: u8,
}

impl Handshake {
  pub fn from_request(request: &Request) -> Result<Self, String> {
    // https://datatracker.ietf.org/doc/html/rfc6455#section-4.2.1

    if let HttpMethod::Get = request.method {
    } else {
      return Err(String::from("HTTP method must be GET"));
    };

    let host = request
      .headers
      .get_header("Host")
      .ok_or(String::from("Host header missing"))?
      .clone();

    let upgrade = request
      .headers
      .get_header("Upgrade")
      .map(|header| header.to_ascii_lowercase())
      .unwrap_or_default();

    if upgrade != "websocket" {
      return Err(String::from(
        "\"Upgrade\" header must be set to \"websocket\" (case insensitive)",
      ));
    };

    let connection = request
      .headers
      .get_header("Connection")
      .map(|header| header.to_ascii_lowercase())
      .unwrap_or_default();

    if connection != "upgrade" {
      return Err(String::from(
        "\"Connection\" header must be set to \"upgrade\" (case insensitive)",
      ));
    };

    let key = request
      .headers
      .get_header("Sec-Websocket-Key")
      .cloned()
      .ok_or(String::from("Websocket key not provided"))?;

    if base_64::decode(&key).len() != 16 {
      return Err(String::from("Decoded key is not 16 bytes"));
    }

    let protocols: Vec<_> =
      header_to_vec(&request.headers, "Sec-Websocket-Protocol");
    // TODO: extract into struct and handle parameters properly
    let extensions: Vec<_> =
      header_to_vec(&request.headers, "Sec-Websocket-Extensions");

    let version = request
      .headers
      .get_header("Sec-Websocket-Version")
      .map(|header| header.parse::<u8>())
      .ok_or(String::from("\"Sec-Websocket-Version\" header must be set"))?
      .map_err(|error| {
        format!("Malformed \"Sec-Websocket-Version\" header: {error}")
      })?;

    Ok(Self {
      resource_name: Self::get_resource_name(&request.uri),
      host,
      key,
      protocols,
      extensions,
      version,
    })
  }

  fn get_resource_name(uri: &Uri) -> String {
    let mut resource_name =
      String::with_capacity(uri.path.len() + uri.fragment.len() + 2);

    if uri.path.is_empty() {
      resource_name.push('/');
    }

    resource_name.push_str(&uri.path);

    if !uri.query.is_empty() {
      resource_name.push('?');
      resource_name.push_str(&uri.query);
    };

    resource_name
  }
}

#[cfg(test)]
mod tests {
  use http::{body::Body, header::Headers};

  use super::*;

  #[test]
  fn resource_name_empty() {
    let uri = Uri::new("").unwrap();
    let resource_name = Handshake::get_resource_name(&uri);

    assert_eq!(resource_name, "/");
  }

  #[test]
  fn resource_name_no_fragment() {
    let uri = Uri::new("/foo").unwrap();
    let resource_name = Handshake::get_resource_name(&uri);

    assert_eq!(resource_name, "/foo");
  }

  #[test]
  fn resource_name_empty_with_fragment() {
    let uri = Uri::new("/?bar").unwrap();
    let resource_name = Handshake::get_resource_name(&uri);

    assert_eq!(resource_name, "/?bar");
  }

  #[test]
  fn resource_name_full() {
    let uri = Uri::new("/foo?bar").unwrap();
    println!("{}", uri.fragment);
    let resource_name = Handshake::get_resource_name(&uri);

    assert_eq!(resource_name, "/foo?bar");
  }

  #[test]
  fn valid_request() {
    // https://datatracker.ietf.org/doc/html/rfc6455#section-1.3
    // https://datatracker.ietf.org/doc/html/rfc6455#section-9.1

    let headers = Headers::from(vec![
      ("Host", "server.example.com"),
      ("Upgrade", "websocket"),
      ("Connection", "Upgrade"),
      ("Sec-Websocket-Key", "dGhlIHNhbXBsZSBub25jZQ=="),
      ("Sec-Websocket-Protocol", "chat, superchat"),
      ("Sec-Websocket-Version", "13"),
      ("Sec-WebSocket-Extensions", "foo, bar; baz=2"),
    ]);

    let request = Request {
      method: HttpMethod::Get,
      uri: Uri::new("/foo?bar").unwrap(),
      headers,
      body: Body::None,
    };

    let handshake = Handshake::from_request(&request).unwrap();

    assert_eq!(
      handshake,
      Handshake {
        resource_name: String::from("/foo?bar"),
        host: String::from("server.example.com"),
        key: String::from("dGhlIHNhbXBsZSBub25jZQ=="),
        protocols: vec![String::from("chat"), String::from("superchat")],
        extensions: vec![String::from("foo"), String::from("bar; baz=2")],
        version: 13
      }
    );
  }
}
