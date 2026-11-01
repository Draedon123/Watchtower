use std::{collections::HashMap, fmt::Display};

pub struct Url {
  pub raw: String,
}

impl Url {
  pub fn new(url: &str) -> Self {
    Self {
      raw: url.to_string(),
    }
  }

  pub fn from_parts(
    protocol: Option<&str>,
    host: &str,
    port: Option<u16>,
    path: &Option<String>,
    query: &Option<HashMap<String, String>>,
  ) -> Self {
    let mut raw = String::new();

    if let Some(protocol) = protocol {
      raw.push_str(protocol);
    }

    raw.push_str(host);

    if let Some(port) = port {
      raw.push(':');
      raw.push_str(&port.to_string());
    }

    if let Some(path) = path {
      raw.push_str(path);
    }

    if let Some(query) = query {
      if query.len() > 0 {
        raw.push('?');

        for (key, value) in query {
          raw.push_str(&format!("{}={}", key, value));
        }
      }
    }

    Url::new(&raw)
  }

  pub fn protocol(&self) -> Result<String, ()> {
    self.protocol_and_rest()?.0.ok_or(())
  }

  pub fn host(&self) -> Result<String, ()> {
    Ok(self.host_and_port()?.0)
  }

  pub fn port(&self) -> Result<u16, ()> {
    match self.host_and_port()?.1 {
      Some(port) => {
        let parsed = port.parse::<u16>();
        match parsed {
          Ok(port) => Ok(port),
          Err(_) => Err(()),
        }
      }
      None => Err(()),
    }
  }

  pub fn path(&self) -> Result<String, ()> {
    Ok(format!("/{}", self.path_and_query()?.0))
  }

  pub fn query(&self) -> Result<HashMap<String, String>, ()> {
    let query = self.path_and_query()?.1;
    let queries = query.split(",").into_iter().filter_map(|query| {
      let (key, value) = query.split_once("=")?;

      Some((key.to_string(), value.to_string()))
    });

    Ok(queries.collect())
  }

  fn protocol_and_rest(&self) -> Result<(Option<String>, String), ()> {
    match self.raw.split_once("://") {
      Some((protocol, rest)) => Ok((Some(protocol.to_string()), rest.to_string())),
      None => Ok((None, self.raw.clone())),
    }
  }

  fn host_port_and_path_query(&self) -> Result<(String, Option<String>), ()> {
    let rest = self.protocol_and_rest()?.1;
    match rest.split_once("/") {
      Some((host_and_port, path_and_query)) => {
        Ok((host_and_port.to_string(), Some(path_and_query.to_string())))
      }
      None => Ok((rest, None)),
    }
  }

  fn host_and_port(&self) -> Result<(String, Option<String>), ()> {
    let split = self.host_port_and_path_query()?.0;
    match split.split_once(":") {
      Some((host, port)) => Ok((host.to_string(), Some(port.to_string()))),
      None => Ok((split.to_string(), None)),
    }
  }

  fn path_and_query(&self) -> Result<(String, String), ()> {
    let split = match self.host_port_and_path_query()?.1 {
      Some(string) => string,
      None => return Ok((String::new(), String::new())),
    };

    match split.split_once("?") {
      Some((path, query)) => Ok((path.to_string(), query.to_string())),
      None => Ok((split.to_string(), String::new())),
    }
  }

  pub fn guess_port(&self) -> Option<u16> {
    match self.protocol().ok()?.as_str() {
      "http" => Some(80),
      _ => None,
    }
  }

  pub fn percent_encode_mut(&mut self) -> Result<(), ()> {
    self.raw = self.percent_encode()?;

    Ok(())
  }

  pub fn percent_encode(&self) -> Result<String, ()> {
    let host = self.host()?;
    let path = self
      .path()
      .ok()
      .map(|path| format!("/{}", Url::encode_uri_component(&path[1..])));
    let query = self.query().ok().map(|query| {
      query
        .iter()
        .map(|(key, value)| {
          (
            Url::encode_uri_component(key),
            Url::encode_uri_component(value),
          )
        })
        .collect()
    });
    let port = self.port().ok();

    let new_url = Url::from_parts(None, &host, port, &path, &query);

    Ok(new_url.raw)
  }

  pub fn percent_decode_mut(&mut self) -> Result<(), String> {
    self.raw = self.percent_decode()?;

    Ok(())
  }

  pub fn percent_decode(&self) -> Result<String, String> {
    Url::decode_uri_component(&self.raw)
  }

  pub fn encode_uri_component(uri: &str) -> String {
    let mut encoded = String::with_capacity(uri.len());

    for byte in uri.bytes() {
      match byte {
        b'A'..=b'Z'
        | b'a'..=b'z'
        | b'0'..=b'9'
        | b'-'
        | b'_'
        | b'.'
        | b'!'
        | b'~'
        | b'*'
        | b'\''
        | b'('
        | b')' => {
          encoded.push(byte as char);
        }
        _ => {
          encoded.push_str(&format!("%{:02X}", byte));
        }
      }
    }

    encoded
  }

  pub fn decode_uri_component(encoded: &str) -> Result<String, String> {
    let mut decoded = Vec::<u8>::with_capacity(encoded.len());
    let bytes = encoded.as_bytes();

    let mut i = 0;

    while i < bytes.len() {
      let byte = bytes[i];

      if byte == b'%' {
        if i + 2 >= bytes.len() {
          return Err(String::from("Incomplete percent encoding"));
        }

        // fetch the two characters after the %
        let hex = str::from_utf8(&bytes[i + 1..=i + 2])
          .map_err(|_| String::from("Invalid percent encoding"))?;
        let byte =
          u8::from_str_radix(hex, 16).map_err(|_| String::from("Invalid percent encoding"))?;

        decoded.push(byte);
        i += 3;
      } else {
        decoded.push(byte);
        i += 1;
      }
    }

    String::from_utf8(decoded).map_err(|_| String::from("Decoded URI component is not valid utf-8"))
  }
}

impl Display for Url {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.raw)
  }
}

#[cfg(test)]
mod tests {
  use std::assert_matches;

  use super::*;

  #[test]
  fn full_url() {
    let url = Url::new("http://foo:123/bar?a=b,c=d");

    assert_eq!(url.protocol().unwrap(), "http");
    assert_eq!(url.host().unwrap(), "foo");
    assert_eq!(url.port().unwrap(), 123);

    let queries = url.query().unwrap();

    assert_eq!(queries.get("a").unwrap(), "b");
    assert_eq!(queries.get("c").unwrap(), "d");
  }

  #[test]
  fn minimal_url() {
    let url = Url::new("foo");

    assert_matches!(url.protocol(), Err(()));
    assert_eq!(url.host().unwrap(), "foo");
    assert_eq!(url.query().unwrap().len(), 0);
  }

  #[test]
  fn percent_encode_decode() {
    let string = "foo.com/$bar🥀";
    let mut url = Url::new(string);

    println!("{:?}", url.raw.bytes());
    url.percent_encode_mut().unwrap();

    assert_eq!(url.raw, "foo.com/%24bar%F0%9F%A5%80");

    println!("{:?}", url.raw.bytes());
    url.percent_decode_mut().unwrap();

    assert_eq!(url.raw, string);
  }
}
