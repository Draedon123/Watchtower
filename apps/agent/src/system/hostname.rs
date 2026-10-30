use std::fs::read_to_string;

pub struct HostnameFetcher {
  hostname_override: Option<String>,
}

impl HostnameFetcher {
  pub fn new() -> HostnameFetcher {
    Self {
      hostname_override: None,
    }
  }

  fn get_hostname_from_os() -> Result<String, String> {
    if let Ok(hostname) = read_to_string("/etc/hostname") {
      Ok(hostname.trim().to_string())
    } else {
      Err(String::from("Could not read from /etc/hostname"))
    }
  }

  pub fn get_hostname(&self) -> Result<String, String> {
    match &self.hostname_override {
      Some(_override) => Ok(_override.to_string()),
      None => Ok(HostnameFetcher::get_hostname_from_os()?),
    }
  }

  pub fn set_override(&mut self, _override: Option<String>) {
    self.hostname_override = _override;
  }
}
