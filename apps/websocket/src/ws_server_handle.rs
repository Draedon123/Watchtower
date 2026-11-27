use tokio::sync::{mpsc, oneshot};

use crate::ws_message::{WsMessage, WsMessageType, WsReply};

#[derive(Clone)]
pub struct WsServerHandle {
  server_tx: mpsc::Sender<WsMessage>,
  client_id: String,
}

impl WsServerHandle {
  pub fn new(server_tx: mpsc::Sender<WsMessage>, client_id: String) -> Self {
    Self {
      server_tx,
      client_id,
    }
  }

  pub async fn send_text(&self, text: String) -> Result<WsReply, String> {
    let (tx, rx) = oneshot::channel::<WsReply>();

    self
      .server_tx
      .send(WsMessage::new(
        WsMessageType::SendText(self.client_id.clone(), text),
        tx,
      ))
      .await
      .map_err(|error| format!("Failed to send message: {error}"))?;

    rx.await.map_err(|_| String::from("Sender dropped"))
  }
}
