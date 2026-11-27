use http::{
  header::Headers, http_connection::HttpConnection,
  http_status_codes::HttpStatusCode, request::Request, response::Response,
};
use tokio::sync::mpsc;

use crate::{
  handshake::Handshake, utils::get_accept_key, ws_message::WsMessage,
  ws_server_handle::WsServerHandle,
};

pub struct Server {
  handle_tx: mpsc::Sender<WsMessage>,
}

impl Server {
  pub fn new() -> (Self, mpsc::Receiver<WsMessage>) {
    let (tx, handle_rx) = mpsc::channel::<WsMessage>(128);

    (Self { handle_tx: tx }, handle_rx)
  }

  pub async fn handle_http_request(
    &self,
    request: &Request,
    connection: &mut HttpConnection,
  ) -> Result<WsServerHandle, String> {
    let handshake = Handshake::from_request(request)?;
    self.accept_handshake(connection, &handshake).await?;

    // TOOD: protect against duplicate keys from malicious clients
    Ok(WsServerHandle::new(self.handle_tx.clone(), handshake.key))
  }

  /// Blocks thread. Must receive the `handle_rx` that was returned from `Server::new`.
  pub async fn start(&self, mut handle_rx: mpsc::Receiver<WsMessage>) {
    loop {
      match handle_rx.recv().await {
        Some(message) => {
          println!(
            "WebSocket Server received message from HTTP Server: {:#?}",
            message.message
          );

          if let Err(error) = message
            .reply_tx
            .send(String::from("Hello from WebSocket Server"))
          {
            eprintln!("WebSocket Server failed to send reply: {error}");
          };
        }
        None => {
          println!("WebSocket Server stopped");
          break;
        }
      }
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
}
