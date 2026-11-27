use http::{
  header::Headers, http_connection::HttpConnection,
  http_status_codes::HttpStatusCode, request::Request, response::Response,
};

use crate::{
  handshake::Handshake, utils::get_accept_key, ws_server_handle::WsServerHandle,
};

pub struct Server {
  handle: WsServerHandle,
}

impl Server {
  pub fn new() -> Self {
    Self {
      handle: WsServerHandle::new(),
    }
  }

  pub async fn handle_http_request(
    &self,
    request: &Request,
    connection: &mut HttpConnection,
  ) -> Result<WsServerHandle, String> {
    let handshake = Handshake::from_request(request)?;
    self.accept_handshake(connection, &handshake).await?;

    Ok(self.handle.clone())
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
}

impl Default for Server {
  fn default() -> Self {
    Self::new()
  }
}
