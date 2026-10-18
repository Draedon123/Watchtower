use std::{env, fs::read_to_string};

use http::request::Request;

use crate::config::Config;

mod config;

fn main() {
  let args: Vec<String> = env::args().collect();
  let config = Config::new(&args);

  let hostname = get_hostname();
  let address = format!("{}", config.watchtower_url);
  let mut request = Request::new(&address);

  request.path = String::from(format!("/set/{}", hostname));
  request.method = http::method::HTTPMethod::Post();
  request.send();
}

fn get_hostname() -> String {
  read_to_string("/etc/hostname").unwrap().trim().to_string()
}
