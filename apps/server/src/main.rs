use config::Config;
use server::Server;
use std::env;

mod config;
mod http;
mod server;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let args: Vec<String> = env::args().collect();
  let config = Config::new(&args);

  Server::new(&config)?;

  Ok(())
}
