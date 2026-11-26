use std::collections::HashMap;

use http::{
  header::Headers, http_connection::HttpConnection,
  http_status_codes::HttpStatusCode, request::Request, response::Response,
};

use crate::{
  client::{Client, Open},
  frame::Frame,
  handshake::Handshake,
  utils::get_accept_key,
  websocket_status_code::WebSocketStatusCode,
};

pub struct Server {
  pub clients: HashMap<String, Client<Open>>,
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
  ) -> Result<Handshake, (String, HttpConnection)> {
    let handshake = Handshake::from_request(request);

    match handshake {
      Ok(handshake) => {
        let result = self.accept_handshake(&mut connection, &handshake).await;

        if let Err(error) = result {
          return Err((error, connection));
        };

        let stream = connection.into_stream();
        self
          .clients
          .insert(handshake.key.clone(), Client::new_open(stream));

        Ok(handshake)
      }
      Err(error) => Err((format!("Rejected handshake: {error}"), connection)),
    }
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

  pub async fn send_to_client(&mut self, client_key: &str, message: String) {
    if let Some(client) = self.clients.get_mut(client_key) {
      let frame = Frame::from_string(message, None);
      if let Err(error) = client.send(frame).await {
        eprintln!("Failed to send frame to client: {error}");
      }
    }
  }

  pub async fn close_client(&mut self, client_key: &str) {
    if let Some(client) = self.clients.remove(client_key)
      && let Err((error, _)) = client
        .close(Some((WebSocketStatusCode::Normal, None)), None)
        .await
    {
      eprintln!("Failed to close client: {}", error);
    }
  }
}

impl Default for Server {
  fn default() -> Self {
    Self::new()
  }
}
