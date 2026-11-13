use http::{client::Client, method::HTTPMethod};
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
  let mut connection = http_client
    .request(
      HTTPMethod::Post(),
      &format!(
        "{}/machine/{hostname}/set/cpu_usage",
        config.watchtower_address
      ),
      None,
    )
    .await?;

  let mut serialised_data = String::new();

  for client in data {
    match client.get_data() {
      Ok((key, value)) => {
        serialised_data.push_str(&format!("{key}\n{value}"));
      }
      Err(error) => eprintln!("{error}"),
    }
  }

  connection.set_content(&serialised_data);
  connection.send().await?;

  Ok(())
}
