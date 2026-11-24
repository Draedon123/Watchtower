use std::marker::PhantomData;

use crypto::base_64;
use http::{
  header::Headers, http_connection::HttpConnectionBorrowed,
  http_method::HttpMethod, http_status_codes::HttpStatusCode, request::Request,
};
use misc::uri::Uri;
use tokio::net::TcpStream;

use crate::utils::{get_accept_key, header_to_vec};

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
}

impl Client<Connecting> {
  pub async fn connect(
    mut self,
    uri: Uri,
  ) -> Result<Client<Open>, Client<Closed>> {
    let mut connection = HttpConnectionBorrowed::new(&mut self.stream);
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
      Ok(response) => match response.status_code {
        HttpStatusCode::Status101 => {
          let upgrade = response.headers.get_header("Upgrade");
          let connection = response.headers.get_header("Connection");
          let accept = response.headers.get_header("Sec-WebSocket-Accept");
          let accepted_extensions =
            header_to_vec(&response.headers, "Sec-WebSocket-Extensions");
          let protocol = response.headers.get_header("Sec-WebSocket-Protocol");

          if !upgrade
            .is_none_or(|upgrade| upgrade.eq_ignore_ascii_case("websocket"))
            || !connection.is_none_or(|connection| {
              connection.eq_ignore_ascii_case("upgrade")
            })
            || accept.is_none_or(|accept| accept.trim() != get_accept_key(&key))
            || accepted_extensions.iter().any(|accepted_extension| {
              extensions.iter().all(|available_extension| {
                accepted_extension != available_extension
              }) || protocol.is_none_or(|accepted_protocol| {
                protocols.iter().all(|available_protocol| {
                  accepted_protocol != available_protocol
                })
              })
            })
          {
            eprintln!("Failed to establish connection");
            Err(Client {
              stream: self.stream,
              state: PhantomData::<Closed>,
            })
          } else {
            Ok(Client {
              stream: self.stream,
              state: PhantomData::<Open>,
            })
          }
        }
        status => {
          eprintln!(
            "Failed to establish connection. Received status code: {status}"
          );

          Err(Client {
            stream: self.stream,
            state: PhantomData::<Closed>,
          })
        }
      },
      Err(error) => {
        eprintln!("Failed to send handshake: {error}");
        Err(Client {
          stream: self.stream,
          state: PhantomData::<Closed>,
        })
      }
    }
  }
}
