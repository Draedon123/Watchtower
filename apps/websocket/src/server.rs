use std::collections::HashMap;

use http::{
  header::Headers, http_connection::HttpConnection,
  http_status_codes::HttpStatusCode, request::Request, response::Response,
};

use crate::{client::Client, handshake::Handshake, utils::get_accept_key};

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
      ("Sec-WebSocket-Accept", &get_accept_key(&handshake.key)),
    ]);

    connection.send_response(&mut response).await
  }
}

impl Default for Server {
  fn default() -> Self {
    Self::new()
  }
}
