use tokio::{io::BufReader, net::TcpStream, sync::mpsc::Sender};

use crate::{frame::Frame, server::ServerMessage};

pub struct ServerClient {
  pub socket: TcpStream,
  sender: Sender<ServerMessage>,
}

impl ServerClient {
  pub fn new(socket: TcpStream, sender: Sender<ServerMessage>) -> Self {
    Self { socket, sender }
  }

  /// Blocks the thread and sends all received messages
  pub async fn send_messages(&mut self) {
    let mut reader = BufReader::new(&mut self.socket);
    loop {
      let frame = Frame::from_buffer(&mut reader).await;

      match frame {
        Ok(frame) => {
          self.sender.send(ServerMessage::Frame(frame)).await.unwrap()
        }
        Err(error) => eprintln!("Failed to read frame: {error}"),
      };
    }
  }
}
