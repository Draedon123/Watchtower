use http::{body::Body, client::Client, method::HTTPMethod};
use watchtower_server::data::DataEntry;

use crate::config::Config;

pub trait Bridge {
  fn get_data(&mut self) -> Result<(String, DataEntry), String>;
}

pub async fn send_to_watchtower<T: Bridge + ?Sized>(
  data: &mut Vec<Box<T>>,
  http_client: &mut Client,
  config: &Config,
  hostname: &str,
) -> Result<(), String> {
  let mut request = http_client
    .request(
      HTTPMethod::Post(),
      &format!("{}/machine/{hostname}/set", config.watchtower_address),
      None,
    )
    .await?;

  let mut serialised_data = String::new();

  for client in data {
    match client.get_data() {
      Ok((key, value)) => {
        serialised_data.push_str(&format!("{key}\n{value}\n"));
      }
      Err(error) => eprintln!("{error}"),
    }
  }

  request.body = Body::from(serialised_data);
  http_client.send_request(&mut request).await?;

  Ok(())
}
