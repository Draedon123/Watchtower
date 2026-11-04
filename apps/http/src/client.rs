use tokio::net::TcpStream;

use crate::{header::Headers, method::HTTPMethod, request::Request, url::Url};

pub struct Client {
  pub default_headers: Headers,

  stream: Option<TcpStream>,
  host: String,
  port: u16,
}

impl Client {
  pub fn new() -> Client {
    Client {
      stream: None,
      host: String::new(),
      port: 0,
      default_headers: Headers::new(),
    }
  }

  pub async fn request(
    &mut self,
    method: HTTPMethod,
    address: &str,
    headers: Option<Headers>,
  ) -> Result<Request<'_>, String> {
    let url = Url::new(address);

    if let Ok(protocol) = url.protocol() {
      if protocol != "http" {
        unimplemented!("Only HTTP requests are supported")
      }
    };

    let host = match url.host() {
      Ok(host) => host,
      Err(_) => return Err(format!("Could not get host from address \"{address}\"")),
    };
    let port = url
      .port()
      .unwrap_or_else(|_| url.guess_port().unwrap_or(80));

    if self.host != host || self.port != port {
      self.host = host.clone();
      self.port = port;

      let stream = match TcpStream::connect((host, port)).await {
        Ok(stream) => stream,
        Err(error) => return Err(error.to_string()),
      };

      self.stream = Some(stream);
    };

    if let Some(stream) = self.stream.as_mut() {
      let mut request = Request::new(stream);
      request.method = method;
      request.path = url;

      request.headers = match headers {
        Some(headers) => Headers::apply_defaults(&self.default_headers, &headers),
        // TODO: don't clone headers every time
        None => self.default_headers.clone(),
      };

      Ok(request)
    } else {
      Err(String::from("No connection made"))
    }
  }
}
