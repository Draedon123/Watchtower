use tokio::{net::TcpStream, sync::mpsc};

use crate::{client_handle::Command, ws_message::WsReply};

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
          println!("Received command {:#?}", command);

          let reply = WsReply::new(String::from("Received command successfully"));
          if let Err(_) = command.reply_tx.send(reply) {
            eprintln!("Failed to send reply");
          }
        }
      }
    }
  }
}

pub enum Event {}
