use crate::{config::Config, route::Routes};
use http::{header::Headers, request::Request, response::Response, url::Url};
use std::{collections::HashMap, pin::Pin, sync::Arc};

use tokio::{
  net::{TcpListener, TcpStream},
  task::JoinSet,
};

pub struct ServerRequest<'a> {
  pub http_request: &'a mut Request<'a>,
  pub slugs: HashMap<String, String>,
}

impl<'a> ServerRequest<'a> {
  pub async fn respond(&mut self, response: &Response) -> Result<(), String> {
    self.http_request.respond(response).await
  }
}

pub struct Server<'a> {
  pub default_response_headers: Headers,
  pub routes: Arc<Routes>,

  config: &'a Config,
  on_connection_close: Option<OnConnectionClose>,
}

pub type OnConnectionCloseFuture = Pin<Box<dyn Future<Output = ()> + Send>>;
pub type OnConnectionClose = Arc<dyn Fn(TcpStream) -> OnConnectionCloseFuture + Send + Sync>;

impl<'a> Server<'a> {
  pub fn new(config: &'a Config) -> Result<Server<'a>, Box<dyn std::error::Error>> {
    let server = Server {
      routes: Arc::new(Routes::new()),
      config,
      default_response_headers: Headers::new(),
      on_connection_close: None,
    };

    Ok(server)
  }

  /// Starts listening on the supplied port. Also blocks the thread.
  pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
    let server_address = format!("127.0.0.1:{}", self.config.port);

    let listener = TcpListener::bind(&server_address).await?;
    let mut connections = JoinSet::new();

    println!("Opened server on {server_address}",);

    loop {
      let (socket, _) = listener.accept().await?;

      let routes = self.routes.clone();
      let default_response_headers = self.default_response_headers.clone();
      let on_connection_close = self.on_connection_close.clone();

      connections.spawn(async move {
        Self::handle_connection(
          socket,
          routes,
          &default_response_headers,
          on_connection_close,
        )
        .await
      });
    }
  }

  pub fn set_default_response_headers(&mut self, headers: Headers) {
    self.default_response_headers = headers;
  }

  pub fn set_on_connection_close<F, Fut>(&mut self, callback: F)
  where
    F: Fn(TcpStream) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
  {
    self.on_connection_close =
      Some(
        Arc::new(move |stream| Box::pin(callback(stream)) as OnConnectionCloseFuture)
          as OnConnectionClose,
      );
  }

  pub fn clear_on_connection_close(&mut self) {
    self.on_connection_close = None;
  }

  async fn handle_connection(
    mut stream: TcpStream,
    routes: Arc<Routes>,
    default_response_headers: &Headers,
    on_connection_close: Option<OnConnectionClose>,
  ) {
    loop {
      let mut http_request = match Request::read_from_buffer(&mut stream).await {
        Ok(Some(request)) => request,
        Ok(None) => break,
        Err(error) => {
          eprintln!("Failed to read incoming TcpStream: {error}");
          continue;
        }
      };

      let mut request_slugs: HashMap<String, String> = HashMap::new();
      let request_method = http_request.method.clone();
      let request_path = match http_request.path.path() {
        Ok(path) => match Url::decode_uri_component(&path) {
          Ok(path) => path,
          Err(_) => {
            eprintln!("Failed to decode request path {}", http_request.path);
            continue;
          }
        },
        Err(_) => {
          eprintln!("Failed to decode request path {}", http_request.path);
          continue;
        }
      };

      let close_connection = match http_request.headers.get_header("Connection") {
        Some(value) => value == "close",
        None => false,
      };

      println!("Received {request_method} request for {request_path}");

      let routes = &routes.routes;
      let route = routes.iter().find(|route| {
        if route.method != request_method {
          false
        } else {
          let match_result = route.matches(&request_path);

          if let Some(slugs) = match_result {
            request_slugs = slugs;
            true
          } else {
            false
          }
        }
      });

      let mut request = ServerRequest {
        http_request: &mut http_request,
        slugs: request_slugs,
      };

      let mut response = Response::new();

      match route {
        Some(route) => {
          response.headers = default_response_headers.clone();

          (route.callback)(&mut request, &mut response).await;
        }
        None => {
          println!("Route not found. Method: {request_method}, Path: {request_path}",);
          let mut response = Response::new();

          response.status_code = http::status_codes::StatusCode::Status404;
          if let Err(error) = request.respond(&response).await {
            eprintln!(
              "Failed to send 404 for {request_method} request for route {request_path}: {error}"
            );
          };
        }
      };

      if close_connection {
        break;
      }
    }

    if let Some(callback) = Option::as_ref(&on_connection_close) {
      callback(stream).await;
    };
  }
}
