use std::{thread::sleep, time};

use http::{client::Client, header::Headers, method::HTTPMethod};

use crate::{config::Config, system::hostname::get_hostname};

pub fn loop_register(config: &Config, client: &mut Client) {
  while let Err(_) = try_register(config, client) {
    sleep(time::Duration::from_millis(1000));
  }
}

pub fn try_register(config: &Config, client: &mut Client) -> Result<(), String> {
  let hostname = match get_hostname() {
    Ok(hostname) => hostname,
    Err(error) => {
      return Err(format!("Failed to get device hostname: {error}"));
    }
  };

  let mut headers = Headers::new();
  headers
    .set_header("Host", &config.watchtower_address)
    .set_header("Authorization", &config.password);

  let mut connection = client.request(
    HTTPMethod::Post(),
    &format!("{}/set/{hostname}", config.watchtower_address),
    Some(headers),
  );

  connection.send()?;

  Ok(())
}
