use std::time::Duration;

use tokio::{net::TcpStream, time::sleep};

use crate::{
  header::Headers, http_connection::HttpConnection, http_method::HttpMethod,
  request::Request, response::Response, url::Url,
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
    address: &str,
    headers: Option<Headers>,
  ) -> Result<Request, String> {
    let url = Url::new(address);

    if let Ok(protocol) = url.protocol() {
      if protocol != "http" {
        unimplemented!("Only HTTP requests are supported")
      }
    };

    let host = url
      .host()
      .map_err(|_| format!("Could not get host from address \"{address}\""))?;
    let port = url
      .port()
      .unwrap_or_else(|_| url.guess_port().unwrap_or(80));

    if self.host != host || self.port != port {
      let stream = match TcpStream::connect((&host[..], port)).await {
        Ok(stream) => {
          self.host = host;
          self.port = port;

          stream
        }
        Err(error) => return Err(error.to_string()),
      };

      self.connection = Some(HttpConnection::new(stream));
    };

    match self.connection {
      Some(_) => {
        let mut request = Request::new();
        request.method = method;
        request.path = url;

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
  ) -> Result<(), String> {
    match self.connection.as_mut() {
      Some(connection) => Ok(connection.send_request(request).await?),
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
