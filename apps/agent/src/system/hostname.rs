use std::fs::read_to_string;

pub fn get_hostname() -> Result<String, String> {
  if let Ok(hostname) = read_to_string("/etc/hostname") {
    Ok(hostname.trim().to_string())
  } else {
    Err(String::from("Could not read from /etc/hostname"))
  }
}
