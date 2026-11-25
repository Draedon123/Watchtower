use std::marker::PhantomData;

use crypto::base_64;
use http::{
  header::Headers, http_connection::HttpConnection, http_method::HttpMethod,
  http_status_codes::HttpStatusCode, request::Request, response::Response,
};
use misc::uri::Uri;
use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::{frame::Frame, utils::get_accept_key};

pub struct Connecting;
pub struct Open;
pub struct Closing;
pub struct Closed;

pub struct Client<State = Connecting> {
  stream: TcpStream,
  state: PhantomData<State>,
}

impl Client {
  pub fn new(stream: TcpStream) -> Self {
    Self {
      stream,
      state: PhantomData::<Connecting>,
    }
  }

  pub fn new_open(stream: TcpStream) -> Client<Open> {
    Client {
      stream,
      state: PhantomData::<Open>,
    }
  }
}

impl Client<Connecting> {
  pub async fn connect(self, uri: Uri) -> Result<Client<Open>, Client<Closed>> {
    let mut connection = HttpConnection::new(self.stream);
    let mut request = Request::new();
    let mut key = [0u8; 16];

    // TODO: use csprng
    rand::fill(&mut key);

    let key = base_64::encode(&key);
    // TODO: protocols + extensions
    let extensions: Vec<String> = Vec::new();
    let protocols: Vec<String> = Vec::new();

    request.headers = Headers::from(vec![
      ("Upgrade", "websocket"),
      ("Connection", "Upgrade"),
      ("Host", &uri.authority),
      ("Sec-WebSocket-Key", &key),
      ("Sec-WebSocket-Extensions", &extensions.join(", ")),
      ("Sec-WebSocket-Protocol", &protocols.join(", ")),
    ]);
    request.uri = uri;
    request.method = HttpMethod::Get;

    let response = connection.send_request(&mut request).await;

    match response {
      Ok(response) => {
        if Self::validate_response(&response, &extensions, &protocols, &key) {
          Ok(Client {
            stream: connection.into_stream(),
            state: PhantomData::<Open>,
          })
        } else {
          eprintln!("Failed to establish connection");
          Err(Client {
            stream: connection.into_stream(),
            state: PhantomData::<Closed>,
          })
        }
      }
      Err(error) => {
        eprintln!("Failed to send handshake: {error}");
        Err(Client {
          stream: connection.into_stream(),
          state: PhantomData::<Closed>,
        })
      }
    }
  }

  // TODO: return Result
  fn validate_response(
    response: &Response,
    available_extensions: &[String],
    available_protocols: &[String],
    key: &str,
  ) -> bool {
    if let HttpStatusCode::Status101 = response.status_code {
    } else {
      return false;
    }

    let upgrade = response.headers.get_header("Upgrade");
    if !upgrade.is_none_or(|upgrade| upgrade.eq_ignore_ascii_case("websocket"))
    {
      return false;
    }

    let connection = response.headers.get_header("Connection");
    if !connection
      .is_none_or(|connection| connection.eq_ignore_ascii_case("upgrade"))
    {
      return false;
    }

    let accept = response.headers.get_header("Sec-WebSocket-Accept");
    if accept.is_none_or(|accept| accept.trim() != get_accept_key(key)) {
      return false;
    }

    let accepted_extensions = &response
      .headers
      .get_headers("Sec-WebSocket-Extensions")
      .unwrap_or_default();
    if accepted_extensions.iter().any(|accepted_extension| {
      available_extensions
        .iter()
        .all(|available_extension| accepted_extension != available_extension)
    }) {
      return false;
    }

    let protocol = response.headers.get_header("Sec-WebSocket-Protocol");
    protocol.is_none_or(|accepted_protocol| {
      available_protocols
        .iter()
        .all(|available_protocol| accepted_protocol != available_protocol)
    })
  }
}

impl Client<Open> {
  pub async fn send(&mut self, frame: Frame) -> Result<(), String> {
    self
      .stream
      .write_all(&frame.into_buffer())
      .await
      .map_err(|error| format!("Failed to write frame to TcpStream: {error}"))
  }
}
