use std::{thread::sleep, time};

use http::{client::Client, method::HTTPMethod};

use crate::{config::Config, system::hostname::HostnameFetcher};

pub fn loop_register(config: &Config, client: &mut Client, hostname_fetcher: &HostnameFetcher) {
  while let Err(_) = try_register(config, client, hostname_fetcher) {
    sleep(time::Duration::from_millis(1000));
  }
}

pub fn try_register(
  config: &Config,
  client: &mut Client,
  hostname_fetcher: &HostnameFetcher,
) -> Result<(), String> {
  let hostname = match hostname_fetcher.get_hostname() {
    Ok(hostname) => hostname,
    Err(error) => {
      return Err(format!("Failed to get device hostname: {error}"));
    }
  };

  let mut connection = client.request(
    HTTPMethod::Post(),
    &format!("{}/set/{hostname}", config.watchtower_address),
    None,
  )?;

  connection.send()?;

  Ok(())
}
