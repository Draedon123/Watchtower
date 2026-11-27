use std::collections::HashMap;

use http::{
  header::Headers, http_connection::HttpConnection,
  http_status_codes::HttpStatusCode, response::Response,
};
use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::{
  frame::Frame, handshake::Handshake, server_client::ServerClient,
  utils::get_accept_key, websocket_status_code::WebSocketStatusCode,
};

pub struct Server {
  clients: HashMap<String, ServerClient>,
  pub sender: Sender<ServerMessage>,
  receiver: Receiver<ServerMessage>,
}

impl Server {
  pub fn new() -> Self {
    let (sender, receiver) = mpsc::channel::<ServerMessage>(8);

    Self {
      clients: HashMap::new(),
      sender,
      receiver,
    }
  }

  pub async fn add_client(
    &mut self,
    handshake: Handshake,
    mut connection: HttpConnection,
  ) -> Result<Handshake, String> {
    self.accept_handshake(&mut connection, &handshake).await?;

    let stream = connection.into_stream();
    let client = ServerClient::new(stream, self.sender.clone());
    self.clients.insert(handshake.key.clone(), client);

    Ok(handshake)
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

  pub async fn send_to_client(&mut self, client_key: &str, message: String) {
    if let Some(client) = self.clients.get_mut(client_key) {
      let frame = Frame::from_string(message, None);
      if let Err(error) = frame.write_to_buffer(&mut client.socket).await {
        eprintln!("Failed to send frame to client: {error}");
      }
    }
  }

  pub async fn close_client(&mut self, client_key: &str) {
    if let Some(mut client) = self.clients.remove(client_key)
      && let Err(error) =
        Frame::close(Some((WebSocketStatusCode::Normal, None)), None)
          .unwrap()
          .write_to_buffer(&mut client.socket)
          .await
    {
      eprintln!("Failed to close client: {}", error);
    }
  }

  pub async fn start(&mut self) {
    while let Some(message) = self.receiver.recv().await {
      match message {
        ServerMessage::Frame(frame) => {
          println!("Received frame: {:#?}", frame);
        }
        ServerMessage::AddClient(handshake, connection) => {
          self.add_client(handshake, connection).await.unwrap();
        }
        ServerMessage::SendToClient(client_key, message) => {
          self.send_to_client(&client_key, message).await;
        }
        ServerMessage::Close(client_key) => {
          self.close_client(&client_key).await;
        }
      }
    }
  }
}

impl Default for Server {
  fn default() -> Self {
    Self::new()
  }
}

#[derive(Debug)]
pub enum ServerMessage {
  Frame(Frame),
  AddClient(Handshake, HttpConnection),
  SendToClient(String, String),
  Close(String),
}
