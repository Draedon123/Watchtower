use std::fs::read_to_string;

pub fn get_hostname() -> String {
  read_to_string("/etc/hostname").unwrap().trim().to_string()
}
