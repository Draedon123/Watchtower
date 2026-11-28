use tokio::{
  io::AsyncWriteExt,
  net::{
    TcpStream,
    tcp::{OwnedReadHalf, OwnedWriteHalf},
  },
  sync::{mpsc, oneshot},
};

use crate::{
  client_handle::{Command, CommandType},
  frame::{Frame, OpCode},
  ws_message::WsReply,
  ws_status_code::WsStatusCode,
};

pub struct ServerClient {
  reader: OwnedReadHalf,
  writer: OwnedWriteHalf,
  command_rx: mpsc::Receiver<Command>,
  event_tx: mpsc::Sender<Event>,
  client_id: String,
}

impl ServerClient {
  pub fn new(
    socket: TcpStream,
    command_rx: mpsc::Receiver<Command>,
    event_tx: mpsc::Sender<Event>,
    client_id: String,
  ) -> Self {
    let (reader, writer) = socket.into_split();

    Self {
      reader,
      writer,
      command_rx,
      event_tx,
      client_id,
    }
  }

  /// Blocks the thread
  pub async fn run(&mut self) {
    loop {
      tokio::select! {
        Some(command) = self.command_rx.recv() => {
          self.run_command(command).await;
        }
        frame = Frame::from_buffer(&mut self.reader) => {
          match frame {
            Ok(frame) => {
              self.send_frame(frame).await;
            }
            // TODO: differentiate between "successful failures"
            Err(error) => {
              eprintln!("Received invalid frame from client: {error}");
              break;
            }
          }
        }
      }
    }
  }

  async fn run_command(&mut self, command: Command) {
    match command.command {
      CommandType::SendText(message) => {
        let frame = Frame::from_string(message, None);

        self.write_frame(Ok(frame), command.reply_tx).await;
      }
      CommandType::SendBinary(bytes) => {
        let frame = Frame::from_bytes(bytes, None);

        self.write_frame(Ok(frame), command.reply_tx).await;
      }
      CommandType::Ping(payload) => {
        let frame = Frame::ping(payload, None);

        self.write_frame(frame, command.reply_tx).await;
      }
      CommandType::Pong(payload) => {
        let frame = Frame::pong(payload, None);

        self.write_frame(frame, command.reply_tx).await;
      }
      CommandType::Close(payload) => {
        let frame = Frame::close(payload, None);

        self.write_frame(frame, command.reply_tx).await;
        self.close().await;
      }
    }
  }

  async fn write_frame(
    &mut self,
    frame: Result<Frame, String>,
    reply_tx: oneshot::Sender<WsReply>,
  ) {
    match frame {
      Ok(frame) => {
        let write_result = frame.write_to_buffer(&mut self.writer).await;

        let reply = match write_result {
          Ok(_) => WsReply::new(String::from("Message sent successfully")),
          Err(error) => {
            WsReply::new(format!("Failed to send message: {error}"))
          }
        };

        Self::try_send_reply(reply, reply_tx);
      }
      Err(error) => {
        let reply = WsReply::new(format!("Failed to send frame: {error}"));
        Self::try_send_reply(reply, reply_tx);
      }
    }
  }

  async fn send_frame(&mut self, frame: Frame) {
    match frame.op_code {
      OpCode::Text => {
        let data = String::from_utf8(frame.payload);

        match data {
          Ok(data) => {
            let event = EventType::Text(data);
            self.send_event(event).await;
          }
          Err(error) => {
            eprintln!("Received invalid utf8 from client: {error}");
          }
        }
      }
      OpCode::Binary => {
        let event = EventType::Binary(frame.payload);
        self.send_event(event).await;
      }
      OpCode::Ping => {
        let event = EventType::Ping(frame.payload);
        self.send_event(event).await;
      }
      OpCode::Pong => {
        let event = EventType::Pong(frame.payload);
        self.send_event(event).await;
      }
      OpCode::Close => {
        let event = if frame.payload.len() >= 2 {
          let status_code = WsStatusCode::from_u16(u16::from_be_bytes([
            frame.payload[0],
            frame.payload[1],
          ]));

          if frame.payload.len() >= 3 {
            let reason = String::from_utf8(frame.payload[2..].to_vec());

            match reason {
              Ok(reason) => EventType::Close(status_code, Some(reason)),
              Err(error) => {
                eprintln!("Received invalid utf8 from client: {error}");
                EventType::Close(status_code, None)
              }
            }
          } else {
            EventType::Close(status_code, None)
          }
        } else {
          EventType::Close(WsStatusCode::NoStatus, None)
        };

        self.send_event(event).await;
        self.close().await;
      }
      _ => todo!(),
    }
  }

  async fn send_event(&self, event: EventType) {
    if self
      .event_tx
      .send(Event::new(event, self.client_id.clone()))
      .await
      .is_err()
    {
      eprintln!("Failed to send event");
    }
  }

  fn try_send_reply(reply: WsReply, reply_tx: oneshot::Sender<WsReply>) {
    if reply_tx.send(reply).is_err() {
      eprintln!("Failed to send reply");
    }
  }

  async fn close(&mut self) {
    self.writer.shutdown().await.unwrap();
  }
}

#[derive(Debug, Clone)]
pub struct Event {
  pub event: EventType,
  pub client_id: String,
}

impl Event {
  pub fn new(event: EventType, client_id: String) -> Self {
    Self { event, client_id }
  }
}

#[derive(Debug, Clone)]
pub enum EventType {
  Text(String),
  Binary(Vec<u8>),
  Ping(Vec<u8>),
  Pong(Vec<u8>),
  Close(WsStatusCode, Option<String>),
}
