use std::collections::HashMap;

use crypto::{base_64, hash::sha1};
use http::{
  header::Headers, http_connection::HttpConnection,
  http_status_codes::HttpStatusCode, request::Request, response::Response,
};

use crate::{client::Client, handshake::Handshake};

const WEBSOCKET_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

pub struct Server {
  pub clients: HashMap<String, Client>,
}

impl Server {
  pub fn new() -> Self {
    Self {
      clients: HashMap::new(),
    }
  }

  #[allow(clippy::result_large_err)]
  pub async fn add_client(
    &mut self,
    request: &Request,
    mut connection: HttpConnection,
  ) -> Result<(), (String, HttpConnection)> {
    let handshake = Handshake::from_request(request);

    match handshake {
      Ok(handshake) => {
        println!("Accepted handshake: {:#?}", handshake);

        let result = self.accept_handshake(&mut connection, &handshake).await;

        if let Err(error) = result {
          return Err((error, connection));
        };

        let stream = connection.into_stream();
        self.clients.insert(handshake.key, Client::new(stream));
      }
      Err(error) => {
        let error = format!("Rejected handshake: {error}");

        println!("{}", error);
        return Err((error, connection));
      }
    };

    Ok(())
  }

  async fn accept_handshake(
    &self,
    connection: &mut HttpConnection,
    handshake: &Handshake,
  ) -> Result<(), String> {
    let mut response = Response::new();

    response.status_code = HttpStatusCode::Status101;
    // TODO: negotiate protocols and extensions
    response.headers = Headers::from(vec![
      ("Upgrade", "websocket"),
      ("Connection", "Upgrade"),
      (
        "Sec-WebSocket-Accept",
        &Self::get_accept_key(&handshake.key),
      ),
    ]);

    connection.send_response(&mut response).await
  }

  fn get_accept_key(key: &str) -> String {
    let mut concatenation =
      String::with_capacity(key.len() + WEBSOCKET_GUID.len());

    concatenation.push_str(key);
    concatenation.push_str(WEBSOCKET_GUID);

    let hashed = sha1(concatenation.as_bytes());

    let bytes = [
      hashed[0].to_be_bytes(),
      hashed[1].to_be_bytes(),
      hashed[2].to_be_bytes(),
      hashed[3].to_be_bytes(),
      hashed[4].to_be_bytes(),
    ];

    // TODO: move into sha1 file
    base_64::encode(&[
      bytes[0][0],
      bytes[0][1],
      bytes[0][2],
      bytes[0][3],
      bytes[1][0],
      bytes[1][1],
      bytes[1][2],
      bytes[1][3],
      bytes[2][0],
      bytes[2][1],
      bytes[2][2],
      bytes[2][3],
      bytes[3][0],
      bytes[3][1],
      bytes[3][2],
      bytes[3][3],
      bytes[4][0],
      bytes[4][1],
      bytes[4][2],
      bytes[4][3],
    ])
  }
}

impl Default for Server {
  fn default() -> Self {
    Self::new()
  }
}
