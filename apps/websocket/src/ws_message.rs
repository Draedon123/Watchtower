use std::fmt::Display;

use tokio::{net::TcpStream, sync::oneshot};

use crate::ws_status_code::WsStatusCode;

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
  /// (Client ID, Socket)
  NewClient(String, TcpStream),
  /// (Client ID, Message)
  SendText(String, String),
  /// (Client ID, Message)
  SendBinary(String, Vec<u8>),
  /// (Client ID, Payload)
  Ping(String, Option<Vec<u8>>),
  /// (Client ID, (Status Code, Reason))
  Close(String, Option<(WsStatusCode, Option<String>)>),
}

#[derive(Debug)]
pub struct WsReply {
  reply: String,
}

impl WsReply {
  pub fn new(reply: String) -> Self {
    Self { reply }
  }

  pub fn get_channel() -> (oneshot::Sender<Self>, oneshot::Receiver<Self>) {
    oneshot::channel::<Self>()
  }
}

impl Display for WsReply {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.reply.fmt(f)
  }
}
