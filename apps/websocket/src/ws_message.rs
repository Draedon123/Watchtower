use tokio::sync::oneshot;

pub type WsReply = String;

#[derive(Debug)]
pub struct WsMessage {
  pub message: WsMessageType,
  pub reply_tx: oneshot::Sender<WsReply>,
}

impl WsMessage {
  pub fn new(
    message: WsMessageType,
    reply_tx: oneshot::Sender<WsReply>,
  ) -> Self {
    Self { message, reply_tx }
  }
}

#[derive(Debug)]
pub enum WsMessageType {
  /// (Client ID, Message)
  SendText(String, String),
}
