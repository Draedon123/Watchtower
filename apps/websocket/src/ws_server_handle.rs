use http::{
  header::Headers, http_connection::HttpConnection,
  http_status_codes::HttpStatusCode, request::Request,
};
use tokio::sync::{mpsc, oneshot};

use crate::{
  handshake::Handshake,
  server_client::Event,
  utils::get_accept_key,
  ws_message::{WsMessage, WsMessageType, WsReply},
  ws_status_code::WsStatusCode,
};

pub struct WsServerHandle {
  server_tx: mpsc::Sender<WsMessage>,
  request_tx: mpsc::Sender<ClientRequest>,
}

impl WsServerHandle {
  pub fn new(
    server_tx: mpsc::Sender<WsMessage>,
    request_tx: mpsc::Sender<ClientRequest>,
  ) -> Self {
    Self {
      server_tx,
      request_tx,
    }
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
    self
      .send_message_to_server(WsMessageType::SendText(client_id, text))
      .await
  }

  pub async fn send_binary(
    &self,
    client_id: String,
    payload: Vec<u8>,
  ) -> Result<WsReply, String> {
    self
      .send_message_to_server(WsMessageType::SendBinary(client_id, payload))
      .await
  }

  pub async fn send_ping(
    &self,
    client_id: String,
    payload: Option<Vec<u8>>,
  ) -> Result<WsReply, String> {
    self
      .send_message_to_server(WsMessageType::Ping(client_id, payload))
      .await
  }

  pub async fn close(
    &self,
    client_id: String,
    payload: Option<(WsStatusCode, Option<String>)>,
  ) -> Result<WsReply, String> {
    self
      .send_message_to_server(WsMessageType::Close(client_id, payload))
      .await
  }

  async fn send_message_to_server(
    &self,
    message: WsMessageType,
  ) -> Result<WsReply, String> {
    let (tx, rx) = WsReply::get_channel();

    self
      .server_tx
      .send(WsMessage::new(message, tx))
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

  pub async fn get_event_receiver(
    &self,
    client_id: String,
  ) -> Option<mpsc::Receiver<Event>> {
    let (reply_tx, reply_rx) =
      oneshot::channel::<Option<mpsc::Receiver<Event>>>();
    let request = ClientRequest {
      client_id,
      reply_tx,
    };

    if let Err(error) = self.request_tx.send(request).await {
      eprintln!("Failed to send client request: {error}");
      return None;
    };

    match reply_rx.await {
      Ok(receiver) => receiver,
      Err(_) => {
        eprintln!("Sender dropped");
        None
      }
    }
  }
}

pub struct ClientRequest {
  pub client_id: String,
  pub reply_tx: oneshot::Sender<Option<mpsc::Receiver<Event>>>,
}
