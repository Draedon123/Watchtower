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

    let key = line_split[0].to_string();
    let value = line_split[1].to_string();

    self.headers.insert(key, value);
  }

  pub fn get_header(&self, key: &str) -> Option<&String> {
    self.headers.get(key)
  }
}

impl Display for Headers {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut headers = self.headers.iter().collect::<Vec<_>>();

    headers.sort_by_key(|entry| entry.0);
    headers.iter().for_each(|entry| {
      f.write_fmt(format_args!("\"{}\": \"{}\",\n", entry.0, entry.1))
        .unwrap()
    });

    Ok(())
  }
}
