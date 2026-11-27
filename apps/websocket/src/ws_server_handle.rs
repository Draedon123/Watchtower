use http::{
  header::Headers, http_connection::HttpConnection,
  http_status_codes::HttpStatusCode, request::Request,
};
use tokio::sync::mpsc;

use crate::{
  handshake::Handshake,
  utils::get_accept_key,
  ws_message::{WsMessage, WsMessageType, WsReply},
};

#[derive(Clone)]
pub struct WsServerHandle {
  server_tx: mpsc::Sender<WsMessage>,
}

impl WsServerHandle {
  pub fn new(server_tx: mpsc::Sender<WsMessage>) -> Self {
    Self { server_tx }
  }

  pub async fn handle_http_request(
    &self,
    id: String,
    request: &Request,
    mut connection: HttpConnection,
  ) -> Result<Option<WsReply>, (String, Box<HttpConnection>)> {
    match Handshake::from_request(request) {
      Ok(handshake) => {
        let result = Self::accept_handshake(&mut connection, &handshake).await;

        match result {
          Ok(_) => {
            let (tx, rx) = WsReply::get_channel();

            if let Err(error) = self
              .server_tx
              .send(WsMessage {
                message: WsMessageType::NewClient(id, connection.into_stream()),
                reply_tx: tx,
              })
              .await
            {
              // TODO: use Err
              return Ok(Some(WsReply::new(error.to_string())));
            };

            Ok(rx.await.ok())
          }
          Err(error) => Err((error, Box::new(connection))),
        }
      }
      Err(error) => Err((error, Box::new(connection))),
    }
  }

  pub async fn send_text(
    &self,
    client_id: String,
    text: String,
  ) -> Result<WsReply, String> {
    let (tx, rx) = WsReply::get_channel();

    self
      .server_tx
      .send(WsMessage::new(WsMessageType::SendText(client_id, text), tx))
      .await
      .map_err(|error| format!("Failed to send message: {error}"))?;

    rx.await.map_err(|_| String::from("Sender dropped"))
  }

  async fn accept_handshake(
    connection: &mut HttpConnection,
    handshake: &Handshake,
  ) -> Result<(), String> {
    let mut response = http::response::Response::new();

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
