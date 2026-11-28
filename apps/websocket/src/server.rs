use std::collections::HashMap;

use tokio::{
  net::TcpStream,
  sync::{mpsc, oneshot},
};

use crate::{
  client_handle::{ClientHandle, Command, CommandType},
  server_client::{Event, EventType, ServerClient},
  ws_message::{WsMessage, WsMessageType, WsReply},
  ws_server_handle::{ClientRequest, WsServerHandle},
};

pub struct Server {
  event_rx: mpsc::Receiver<Event>,
  event_tx: mpsc::Sender<Event>,
  handle_rx: mpsc::Receiver<WsMessage>,
  handle_tx: mpsc::Sender<WsMessage>,
  client_request_rx: mpsc::Receiver<ClientRequest>,
  client_request_tx: mpsc::Sender<ClientRequest>,
  private_client_handles: HashMap<String, ClientHandle>,
  public_client_handles: HashMap<String, mpsc::Sender<Event>>,
}

impl Server {
  pub fn new() -> Self {
    let (handle_tx, handle_rx) = mpsc::channel::<WsMessage>(128);
    let (event_tx, event_rx) = mpsc::channel::<Event>(32);
    let (client_request_tx, client_request_rx) =
      mpsc::channel::<ClientRequest>(32);

    Self {
      event_rx,
      event_tx,
      handle_tx,
      handle_rx,
      client_request_rx,
      client_request_tx,
      private_client_handles: HashMap::new(),
      public_client_handles: HashMap::new(),
    }
  }

  pub fn get_handle(&self) -> WsServerHandle {
    WsServerHandle::new(self.handle_tx.clone(), self.client_request_tx.clone())
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

        request = self.client_request_rx.recv() => {
          match request {
            Some(request) => {
              self.handle_client_request(request).await;
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

  async fn handle_event(&mut self, event: Event) {
    if !self.private_client_handles.contains_key(&event.client_id) {
      eprintln!("Client {} not found", event.client_id);
      return;
    };

    match event.event {
      EventType::Close(status_code, reason) => {
        self.private_client_handles.remove(&event.client_id);

        println!(
          "Closed connection \"{:#?}\". Status: {}. Reason: {:#?}",
          status_code, event.client_id, reason
        );

        if let Some(tx) = self.public_client_handles.remove(&event.client_id)
          && let Err(error) = tx
            .send(Event {
              event: EventType::Close(status_code, reason),
              client_id: event.client_id,
            })
            .await
        {
          eprintln!("Failed to forward event from server: {error}");
        }
      }
      _ => {
        let client_id = event.client_id.clone();

        if let Some(tx) = self.public_client_handles.get(&client_id)
          && let Err(error) = tx.send(event).await
        {
          eprintln!("Failed to forward event from server: {error}");
        };
      }
    };
  }

  async fn send_command_to_client(
    &mut self,
    client_id: String,
    command: CommandType,
    reply_tx: oneshot::Sender<WsReply>,
  ) {
    let client_handle = self.private_client_handles.get(&client_id);

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

    let mut client = ServerClient::new(
      socket,
      command_rx,
      self.event_tx.clone(),
      client_id.clone(),
    );

    // TODO: handle dupliate client_id's
    let client_handle = ClientHandle::new(command_tx);
    self.private_client_handles.insert(client_id, client_handle);

    tokio::spawn(async move { client.run().await });
  }

  async fn handle_client_request(&mut self, request: ClientRequest) {
    if !self.private_client_handles.contains_key(&request.client_id) {
      if request.reply_tx.send(None).is_err() {
        eprintln!("Sender dropped");
      }

      return;
    }

    // TODO: handle duplicate client_id's
    let (client_tx, client_rx) = mpsc::channel::<Event>(32);
    self
      .public_client_handles
      .insert(request.client_id, client_tx);

    if request.reply_tx.send(Some(client_rx)).is_err() {
      eprintln!("Sender dropped");
    }
  }
}

impl Default for Server {
  fn default() -> Self {
    Self::new()
  }
}
