use tokio::sync::{mpsc, oneshot};

use crate::ws_message::WsReply;

pub struct ClientHandle {
  tx: mpsc::Sender<Command>,
}

impl ClientHandle {
  pub fn new(tx: mpsc::Sender<Command>) -> Self {
    Self { tx }
  }

  pub async fn send_command(&self, command: Command) -> Result<(), String> {
    self
      .tx
      .send(command)
      .await
      .map_err(|error| format!("Failed to send command to client: {error}"))?;

    Ok(())
  }
}

#[derive(Debug)]
pub struct Command {
  pub command: CommandType,
  pub reply_tx: oneshot::Sender<WsReply>,
}

impl Command {
  pub fn new(command: CommandType, reply_tx: oneshot::Sender<WsReply>) -> Self {
    Self { command, reply_tx }
  }
}

#[derive(Debug)]
pub enum CommandType {
  Send(Vec<u8>),
}
