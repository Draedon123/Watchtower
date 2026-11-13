use std::{thread::sleep, time};

use http::{client::Client, method::HTTPMethod};

use crate::config::Config;

pub async fn loop_register(config: &Config, client: &mut Client, hostname: &str) {
  while let Err(_) = try_register(config, client, hostname).await {
    sleep(time::Duration::from_millis(1000));
  }
}

pub async fn try_register(
  config: &Config,
  client: &mut Client,
  hostname: &str,
) -> Result<(), String> {
  let mut connection = client
    .request(
      HTTPMethod::Post(),
      &format!("{}/machine/register/{hostname}", config.watchtower_address),
      None,
    )
    .await?;

  connection.send().await?;

  Ok(())
}
