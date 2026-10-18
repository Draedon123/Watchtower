use config::Config;
use server::Server;
use std::{
  env,
  io::{self, BufRead},
  thread,
};

mod config;
mod server;

use http::method::HTTPMethod;
use http::status_codes::StatusCode;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let args: Vec<String> = env::args().collect();
  let config = Config::new(&args);

  thread::spawn(move || {
    let mut server = Server::new(&config).unwrap();

    server.route("/temp", HTTPMethod::Post(), |mut request, mut response| {
      response.status_code = StatusCode::Status200;
      response.set_content("50.2");

      request.respond(&response);
    });

    server.start().unwrap();
  });

  let stdin = io::stdin();

  for line in stdin.lock().lines() {
    let line = line.unwrap_or(String::new());

    if line == "close" {
      break;
    }
  }

  Ok(())
}
