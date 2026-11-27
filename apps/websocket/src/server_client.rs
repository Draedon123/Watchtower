use tokio::{
  net::TcpStream,
  sync::{mpsc, oneshot},
};

use crate::{
  client_handle::{Command, CommandType},
  frame::Frame,
  ws_message::WsReply,
};

pub struct ServerClient {
  socket: TcpStream,
  command_rx: mpsc::Receiver<Command>,
  event_tx: mpsc::Sender<Event>,
}

impl ServerClient {
  pub fn new(
    socket: TcpStream,
    command_rx: mpsc::Receiver<Command>,
    event_tx: mpsc::Sender<Event>,
  ) -> Self {
    Self {
      socket,
      command_rx,
      event_tx,
    }
  }

  /// Blocks the thread
  pub async fn run(&mut self) {
    loop {
      tokio::select! {
        Some(command) = self.command_rx.recv() => {
          self.run_command(command).await;
        }
      }
    }
  }

  async fn run_command(&mut self, command: Command) {
    match command.command {
      CommandType::SendText(message) => {
        let frame = Frame::from_string(message, None);

        self.send_frame(frame, command.reply_tx).await;
      }
      CommandType::SendBinary(bytes) => {
        let frame = Frame::from_bytes(bytes, None);

        self.send_frame(frame, command.reply_tx).await;
      }
    }
  }

  async fn send_frame(
    &mut self,
    frame: Frame,
    reply_tx: oneshot::Sender<WsReply>,
  ) {
    let write_result = frame.write_to_buffer(&mut self.socket).await;

    let reply = match write_result {
      Ok(_) => WsReply::new(String::from("Message sent successfully")),
      Err(error) => WsReply::new(format!("Failed to send message: {error}")),
    };

    if reply_tx.send(reply).is_err() {
      eprintln!("Failed to send reply");
    }
  }
}

pub enum Event {}
