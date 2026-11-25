use std::fmt::Display;

use regex::{Match, regex};

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Uri {
  pub scheme: String,
  pub authority: String,
  pub path: String,
  pub query: String,
  pub fragment: String,
}

#[derive(Debug)]
pub struct UriParseError;

impl Display for UriParseError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Failed to parse raw uri")
  }
}

impl Uri {
  pub fn new(uri: &str) -> Result<Self, UriParseError> {
    // https://datatracker.ietf.org/doc/html/rfc3986#appendix-B
    let captures =
      regex!(r"^(([^:/?#]+):)?(//([^/?#]*))?([^?#]*)(\?([^#]*))?(#(.*))?")
        .captures(uri)
        .ok_or(UriParseError)?;

    let scheme = Self::option_match_to_string(captures.get(2));
    let authority = Self::option_match_to_string(captures.get(4));
    let path = Self::option_match_to_string(captures.get(5));
    let query = Self::option_match_to_string(captures.get(7));
    let fragment = Self::option_match_to_string(captures.get(9));

    Ok(Self {
      scheme,
      authority,
      path,
      query,
      fragment,
    })
  }

  fn option_match_to_string(regex_match: Option<Match<'_>>) -> String {
    regex_match
      .map(|regex_match| regex_match.as_str())
      .unwrap_or("")
      .to_owned()
  }

  fn split_authority(&self) -> (Option<&str>, &str, Option<u16>) {
    let split = self.authority.split_once('&');
    let (user_info, rest) = match split {
      Some((user_info, rest)) => (Some(user_info), rest),
      None => (None, &self.authority as &str),
    };

    let (host, rest) = match rest.split_once(']') {
      // IPv6 or IPvFuture. Returns `&host[1..]` since there is still the leading '['
      Some((host, rest)) => (&host[1..], rest),
      None => match rest.split_once(':') {
        Some(split) => split,
        None => (rest, ""),
      },
    };

    let port = rest.trim_prefix(':').parse::<u16>().ok();

    (user_info, host, port)
  }

  pub fn get_user_info(&self) -> Option<&str> {
    self.split_authority().0
  }

  pub fn get_host(&self) -> &str {
    self.split_authority().1
  }

  pub fn get_port(&self) -> Option<u16> {
    self.split_authority().2
  }

  pub fn is_relative(&self) -> bool {
    self.scheme.is_empty()
  }

  pub fn with_path(&self, path: String) -> Self {
    let mut uri = self.clone();
    uri.path = path;

    uri
  }

  pub fn percent_encode(uri: &str) -> String {
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

  pub fn percent_decode(encoded: &str) -> Result<String, String> {
    let mut decoded = Vec::<u8>::with_capacity(encoded.len());
    let bytes = encoded.as_bytes();

    let mut i = 0;

    while i < bytes.len() {
      let byte = bytes[i];

      if byte == b'%' {
        if i + 2 >= bytes.len() {
          return Err(String::from(
            "Incomplete percent encoding: trailing/incomplete %",
          ));
        }

        // fetch the two characters after the %
        let hex = str::from_utf8(&bytes[i + 1..=i + 2]).map_err(|error| {
          format!("Invalid percent encoding: invalid utf8 code: {error}")
        })?;
        let byte = u8::from_str_radix(hex, 16).map_err(|error| {
          format!("Invalid percent encoding: invalid hex \"{hex}\": {error}")
        })?;

        decoded.push(byte);
        i += 3;
      } else {
        decoded.push(byte);
        i += 1;
      }
    }

    String::from_utf8(decoded).map_err(|error| {
      format!("Decoded URI component is not valid utf-8: {error}")
    })
  }
}

impl Display for Uri {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let hier_part = if self.authority.is_empty() {
      &self.path
    } else {
      &format!("//{}{}", self.authority, self.path)
    };

    let query = if self.query.is_empty() {
      String::new()
    } else {
      format!("?{}", self.query)
    };

    let fragment = if self.fragment.is_empty() {
      String::new()
    } else {
      format!("#{}", self.fragment)
    };

    if self.is_relative() {
      write!(f, "{hier_part}{query}{fragment}")
    } else {
      write!(f, "{}:{hier_part}{query}{fragment}", self.scheme)
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn full_uri() {
    let raw = "http://www.ics.uci.edu:80/pub/ietf/uri/?foo=bar&baz#Related";
    let uri = Uri::new(raw).expect("Provided URI is valid");

    assert_eq!(uri.scheme, "http");
    assert_eq!(uri.authority, "www.ics.uci.edu:80");
    assert_eq!(uri.path, "/pub/ietf/uri/");
    assert_eq!(uri.query, "foo=bar&baz");
    assert_eq!(uri.fragment, "Related");

    assert_eq!(uri.get_port(), Some(80));
    assert_eq!(uri.to_string(), raw);
  }

  #[test]
  fn minimal_uri() {
    let raw = "http://foo";
    let uri = Uri::new(raw).expect("Provided URI is valid");

    assert_eq!(uri.scheme, "http");
    assert_eq!(uri.authority, "foo");
    assert_eq!(uri.path, "");
    assert_eq!(uri.query, "");
    assert_eq!(uri.fragment, "");

    assert_eq!(uri.get_port(), None);
    assert_eq!(uri.to_string(), raw);
  }

  #[test]
  fn minimal_relative_uri() {
    let path = "/foo";
    let uri = Uri::new(path).expect("Provided URI is valid");

    assert_eq!(uri.scheme, "");
    assert_eq!(uri.authority, "");
    assert_eq!(uri.path, "/foo");
    assert_eq!(uri.query, "");
    assert_eq!(uri.fragment, "");

    assert!(uri.is_relative());
    assert_eq!(uri.to_string(), path);
  }

  #[test]
  fn percent_encode_decode() {
    let string = "foo.com/$bar🥀";
    let encoded = Uri::percent_encode(string);
    let decoded = Uri::percent_decode(&encoded)
      .expect("Decoding correctly encoded values should not fail");

    assert_eq!(encoded, "foo.com%2F%24bar%F0%9F%A5%80");
    assert_eq!(decoded, string);
  }
}
