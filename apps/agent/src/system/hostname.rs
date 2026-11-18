use std::fs::read_to_string;

pub fn get_hostname() -> Result<String, String> {
  read_to_string("/etc/hostname")
    .map(|hostname| hostname.trim().to_string())
    .map_err(|error| format!("Could not read from /etc/hostname: {error}"))
}
