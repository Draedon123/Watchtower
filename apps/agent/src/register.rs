use std::{thread::sleep, time};

use http::request::Request;

use crate::{config::Config, system::hostname::get_hostname};

pub fn loop_register(config: &Config) {
  while let Err(_) = try_register(&config) {
    sleep(time::Duration::from_millis(1000));
  }
}

pub fn try_register(config: &Config) -> Result<(), ()> {
  let mut connection = Request::new_auto_handle_errors(&config.watchtower_url);
  let hostname = get_hostname();

  if let None = connection.stream {
    eprintln!("Failed to register device with watchtower");
    return Err(());
  };

  connection.path = String::from(format!("/set/{}", hostname));
  connection.method = http::method::HTTPMethod::Post();
  connection
    .headers
    .set_header("Host", &config.watchtower_url);
  connection.set_content(&config.password);
  connection.send();

  Ok(())
}
