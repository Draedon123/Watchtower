use std::collections::HashMap;

#[derive(Clone)]
pub struct Headers {
  headers: HashMap<String, String>,
}

impl Headers {
  pub fn new() -> Self {
    Self {
      headers: HashMap::new(),
    }
  }

  pub fn add_header_line(&mut self, line: &str) -> Result<(), String> {
    let (key, value) = match line.split_once(":") {
      Some((key, value)) => (key.trim().to_lowercase(), value.trim()),
      None => return Err(format!("Received malformed header line: {line}")),
    };

    self.set_header(&key, value);
    Ok(())
  }

  pub fn get_header(&self, key: &str) -> Option<&String> {
    self.headers.get(&key.to_lowercase())
  }

  pub fn set_header(&mut self, key: &str, value: &str) -> &mut Self {
    self
      .headers
      .insert(key.to_string().to_lowercase(), value.to_string());

    self
  }

  pub fn combine(source: &Self, overwrites: &Self) -> Self {
    let mut copy = source.clone();

    for (key, value) in &overwrites.headers {
      copy.set_header(key, value);
    }

    copy
  }

  pub fn serialise(&self) -> String {
    self
      .headers
      .iter()
      .map(|entry| format!("{}: {}\n", entry.0, entry.1))
      .collect::<Vec<_>>()
      .join("")
  }
}

impl From<Vec<(&str, &str)>> for Headers {
  fn from(value: Vec<(&str, &str)>) -> Self {
    let mut headers = Self::new();

    for (key, value) in value {
      headers.set_header(key, value);
    }

    headers
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_defaults() {
    let mut default = Headers::new();
    default.set_header("Foo", "Bar").set_header("a", "b");

    let mut overwrites = Headers::new();
    overwrites.set_header("Foo", "Baz");

    let defaults_applied = Headers::combine(&default, &overwrites);

    assert_eq!(
      defaults_applied.get_header("Foo"),
      Some(&String::from("Baz"))
    );

    assert_eq!(defaults_applied.get_header("a"), Some(&String::from("b")));
  }
}
