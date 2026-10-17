use std::{collections::HashMap, fmt::Display};

pub struct Headers {
  headers: HashMap<String, String>,
}

impl Headers {
  pub fn new() -> Headers {
    Headers {
      headers: HashMap::new(),
    }
  }

  pub fn add_header_line(&mut self, line: &str) {
    let line_split: Vec<_> = line.split(":").map(|string| string.trim()).collect();

    let key = line_split[0];
    let value = line_split[1];

    self.set_header(key, value);
  }

  pub fn get_header(&self, key: &str) -> Option<&String> {
    self.headers.get(key)
  }

  pub fn set_header(&mut self, key: &str, value: &str) {
    self.headers.insert(key.to_string(), value.to_string());
  }
}

impl Display for Headers {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let stringified = self
      .headers
      .iter()
      .map(|entry| format!("{}: {}\n", entry.0, entry.1))
      .collect::<Vec<_>>()
      .join("");

    f.write_str(&stringified)?;

    Ok(())
  }
}
