use std::collections::HashMap;

use http::request::Request;
use tokio::net::TcpStream;

use crate::{client::Client, handshake::Handshake};

pub struct Server {
  pub clients: HashMap<String, Client>,
}

impl Server {
  pub fn new() -> Self {
    Self {
      clients: HashMap::new(),
    }
  }

  pub fn add_client(
    &mut self,
    request: &Request,
    connection: TcpStream,
  ) -> Result<(), (String, TcpStream)> {
    let handshake = Handshake::from_request(request);

    match handshake {
      Ok(handshake) => {
        println!("Accepted handshake: {:#?}", handshake);
        self.clients.insert(handshake.key, Client::new(connection));
      }
      Err(error) => {
        let error = format!("Rejected handshake: {error}");
        println!("{}", error);
        return Err((error, connection));
      }
    };

    Ok(())
  }
}

impl Default for Server {
  fn default() -> Self {
    Self::new()
  }
}
