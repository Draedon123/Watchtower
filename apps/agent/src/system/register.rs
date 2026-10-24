use std::{thread::sleep, time};

use http::request::Request;

use crate::{config::Config, system::hostname::get_hostname};

pub fn loop_register(config: &Config) {
  while let Err(_) = try_register(&config) {
    sleep(time::Duration::from_millis(1000));
  }
}

pub fn try_register(config: &Config) -> Result<(), String> {
  let mut connection = Request::new_auto_handle_errors(&config.watchtower_address);

  if let None = connection.stream {
    return Err(String::from("Failed to register device with watchtower"));
  };

  let hostname = match get_hostname() {
    Ok(hostname) => hostname,
    Err(error) => {
      return Err(format!("Failed to get device hostname: {error}"));
    }
  };

  connection.path = String::from(format!("/set/{hostname}"));
  connection.method = http::method::HTTPMethod::Post();
  connection
    .headers
    .set_header("Host", &config.watchtower_address)
    .set_header("Authorization", &config.password);
  connection.set_content(&config.password);
  connection.send()?;

  Ok(())
}
