use std::{thread::sleep, time};

use http::{client::Client, http_method::HttpMethod};

use crate::config::Config;

pub async fn loop_register(
  config: &Config,
  client: &mut Client,
  hostname: &str,
) {
  while let Err(error) = try_register(config, client, hostname).await {
    eprintln!("Failed to register {hostname}: {error}");
    sleep(time::Duration::from_millis(1000));
  }

  println!("Successfully registered {hostname}");
}

pub async fn try_register(
  config: &Config,
  client: &mut Client,
  hostname: &str,
) -> Result<(), String> {
  let mut request = client
    .request(
      HttpMethod::Post,
      config
        .watchtower_uri
        .with_path(format!("/machine/register/{hostname}")),
      None,
    )
    .await?;

  if let Err(error) = client.send_request(&mut request).await {
    client.reconnect_loop(1000).await;
    return Err(error);
  };

  Ok(())
}
