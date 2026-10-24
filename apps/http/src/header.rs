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

  pub fn add_header_line(&mut self, line: &str) -> Result<(), String> {
    let line_split = if let Some(split) = line.split_once(":") {
      (split.0.trim(), split.1.trim())
    } else {
      return Err(format!("Received malformed header line: {line}"));
    };

    let key = &line_split.0.to_lowercase();
    let value = line_split.1;

    self.set_header(key, value);
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
