use std::collections::HashMap;

use tokio::{
  net::TcpStream,
  sync::{mpsc, oneshot},
};

use crate::{
  client_handle::{ClientHandle, Command, CommandType},
  server_client::{Event, ServerClient},
  ws_message::{WsMessage, WsMessageType, WsReply},
  ws_server_handle::WsServerHandle,
};

pub struct Server {
  event_rx: mpsc::Receiver<Event>,
  event_tx: mpsc::Sender<Event>,
  handle_rx: mpsc::Receiver<WsMessage>,
  handle_tx: mpsc::Sender<WsMessage>,
  client_handles: HashMap<String, ClientHandle>,
}

impl Server {
  pub fn new() -> Self {
    let (handle_tx, handle_rx) = mpsc::channel::<WsMessage>(128);
    let (event_tx, event_rx) = mpsc::channel::<Event>(32);

    Self {
      event_rx,
      event_tx,
      handle_tx,
      handle_rx,
      client_handles: HashMap::new(),
    }
  }

  pub fn get_handle(&self) -> WsServerHandle {
    WsServerHandle::new(self.handle_tx.clone())
  }

  /// Blocks thread
  pub async fn start(&mut self) {
    loop {
      tokio::select! {
        message = self.handle_rx.recv() => {
          match message {
            Some(message) => {
              self.handle_message(message).await;
            }
            None => {
              println!("WebSocket Server stopped");
              break;
            }
          }
        }

        event = self.event_rx.recv() => {
          match event {
            Some(event) => {
              self.handle_event(event).await;
            }
            None => {
              println!("WebSocket Server stopped");
              break;
            }
          }
        }
      }
    }
  }

  async fn handle_message(&mut self, message: WsMessage) {
    match message.message {
      WsMessageType::NewClient(client_id, socket) => {
        self.add_client(client_id, socket);
      }
      WsMessageType::SendText(client_id, payload) => {
        self
          .send_command_to_client(
            client_id,
            CommandType::SendText(payload),
            message.reply_tx,
          )
          .await;
      }
      WsMessageType::SendBinary(client_id, payload) => {
        self
          .send_command_to_client(
            client_id,
            CommandType::SendBinary(payload),
            message.reply_tx,
          )
          .await;
      }
      WsMessageType::Ping(client_id, payload) => {
        self
          .send_command_to_client(
            client_id,
            CommandType::Ping(payload),
            message.reply_tx,
          )
          .await;
      }
      WsMessageType::Close(client_id, payload) => {
        self
          .send_command_to_client(
            client_id,
            CommandType::Close(payload),
            message.reply_tx,
          )
          .await;
      }
    }
  }

  async fn handle_event(&self, event: Event) {
    println!("Received event from client: {:#?}", event);
  }

  async fn send_command_to_client(
    &mut self,
    client_id: String,
    command: CommandType,
    reply_tx: oneshot::Sender<WsReply>,
  ) {
    let client_handle = self.client_handles.get(&client_id);

    match client_handle {
      Some(client_handle) => {
        let command = Command::new(command, reply_tx);

        // TODO: error handling
        let _ = client_handle.send_command(command).await;
      }
      None => {
        let reply = WsReply::new(String::from("Client not found"));
        let reply_result = reply_tx.send(reply);

        if reply_result.is_err() {
          eprintln!("WebSocket Server failed to send reply");
        }
      }
    }
  }

  fn add_client(&mut self, client_id: String, socket: TcpStream) {
    let (command_tx, command_rx) = mpsc::channel::<Command>(32);

    let mut client =
      ServerClient::new(socket, command_rx, self.event_tx.clone());

    // TODO: handle dupliate client_id's
    let client_handle = ClientHandle::new(command_tx);
    self.client_handles.insert(client_id, client_handle);

    tokio::spawn(async move { client.run().await });
  }
}

impl Default for Server {
  fn default() -> Self {
    Self::new()
  }
}
