use std::collections::HashMap;

pub struct Url {
  raw: String,
}

impl Url {
  pub fn new(url: &str) -> Self {
    Self {
      raw: url.to_string(),
    }
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

  fn host_port_and_path_query(&self) -> Result<(String, String), ()> {
    match self.protocol_and_rest()?.1.split_once("/") {
      Some((host_and_port, path_and_query)) => {
        Ok((host_and_port.to_string(), path_and_query.to_string()))
      }
      None => return Err(()),
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
    let split = self.host_port_and_path_query()?.1;
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
}
