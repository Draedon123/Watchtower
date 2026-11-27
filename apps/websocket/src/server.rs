use std::collections::HashMap;

use tokio::{net::TcpStream, sync::mpsc};

use crate::{
  server_client::ServerClient,
  ws_message::{WsMessage, WsReply},
  ws_server_handle::WsServerHandle,
};

pub struct Server {
  handle_tx: mpsc::Sender<WsMessage>,
  clients: HashMap<String, ServerClient>,
}

impl Server {
  pub fn new() -> (Self, mpsc::Receiver<WsMessage>) {
    let (handle_tx, handle_rx) = mpsc::channel::<WsMessage>(128);

    (
      Self {
        handle_tx,
        clients: HashMap::new(),
      },
      handle_rx,
    )
  }

  pub fn get_handle(&self, id: String) -> WsServerHandle {
    WsServerHandle::new(self.handle_tx.clone(), id)
  }

  fn initialise_new_client(&self, socket: TcpStream) {
    let client = ServerClient::new(socket);

    tokio::spawn(async move { client.run().await });
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
            .send(WsReply::new(String::from("Hello from WebSocket Server")))
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
}
