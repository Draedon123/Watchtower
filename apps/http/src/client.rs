use std::time::Duration;

use misc::uri::Uri;
use tokio::{net::TcpStream, time::sleep};

use crate::{
  header::Headers, http_connection::HttpConnection, http_method::HttpMethod,
  request::Request, response::Response,
};

pub struct Client {
  pub default_headers: Headers,

  pub connection: Option<HttpConnection>,
  host: String,
  port: u16,
}

impl Client {
  pub fn new() -> Self {
    Self {
      connection: None,
      host: String::new(),
      port: 0,
      default_headers: Headers::new(),
    }
  }

  pub async fn request(
    &mut self,
    method: HttpMethod,
    uri: Uri,
    headers: Option<Headers>,
  ) -> Result<Request, String> {
    if uri.scheme != "http" {
      unimplemented!("Unrecognised scheme \"{}\"", uri.scheme);
    };

    let port = uri.get_port().unwrap_or(80);

    if self.host != uri.get_host() || self.port != port {
      let stream = TcpStream::connect((uri.get_host(), port))
        .await
        .map_err(|error| format!("TcpStream::connect failed: {error}"))?;

      self.host = uri.get_host().to_owned();
      self.port = port;
      self.connection = Some(HttpConnection::new(stream));
    };

    match self.connection {
      Some(_) => {
        let mut request = Request::new();
        request.method = method;
        request.uri = uri;

        request.headers = match headers {
          Some(headers) => Headers::combine(&self.default_headers, &headers),
          // TODO: don't clone headers every time
          None => self.default_headers.clone(),
        };

        Ok(request)
      }
      None => Err(String::from("No connection made")),
    }
  }

  pub async fn send_request(
    &mut self,
    request: &mut Request,
  ) -> Result<Response, String> {
    match self.connection.as_mut() {
      Some(connection) => Ok(connection.send_request(request).await?),
      None => Err(String::from("Could not send request: disconnected")),
    }
  }

  /// Does not wait for or read response
  pub async fn send_request_rude(
    &mut self,
    request: &mut Request,
  ) -> Result<(), String> {
    match self.connection.as_mut() {
      Some(connection) => Ok(connection.send_request_rude(request).await?),
      None => Err(String::from("Could not send request: disconnected")),
    }
  }

  pub async fn send_response(
    &mut self,
    response: &mut Response,
  ) -> Result<(), String> {
    match self.connection.as_mut() {
      Some(connection) => Ok(connection.send_response(response).await?),
      None => Err(String::from("Could not send request: disconnected")),
    }
  }

  pub async fn reconnect_loop(&mut self, timeout_ms: u64) {
    loop {
      if let Err(error) = self.try_reconnect_once().await {
        eprintln!(
          "Failed to reconnect to {}:{}. Error: {error}",
          self.host, self.port
        );
      };

      sleep(Duration::from_millis(timeout_ms)).await;
    }
  }

  pub async fn try_reconnect_once(&mut self) -> Result<(), std::io::Error> {
    let stream = TcpStream::connect((&self.host[..], self.port)).await?;
    self.connection = Some(HttpConnection::new(stream));

    Ok(())
  }
}

impl Default for Client {
  fn default() -> Self {
    Self::new()
  }
}
