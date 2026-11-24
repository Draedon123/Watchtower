use crate::{
  config::Config,
  route::{RouteResult, Routes},
};
use http::{
  header::Headers, http_connection::HttpConnectionOwned, request::Request,
  response::Response,
};
use misc::uri::Uri;
use std::{collections::HashMap, pin::Pin, sync::Arc};

use tokio::{
  net::{TcpListener, TcpStream},
  task::JoinSet,
};

pub struct ServerRequest {
  pub http_request: Request,
  pub slugs: HashMap<String, String>,
  pub request_id: u64,
}

pub struct Server<'a> {
  pub default_response_headers: Headers,
  pub routes: Arc<Routes>,

  config: &'a Config,
  on_connection_close: Option<OnConnectionClose>,
  request_id: u64,
}

pub type OnConnectionCloseFuture = Pin<Box<dyn Future<Output = ()> + Send>>;
pub type OnConnectionClose =
  Arc<dyn Fn(HttpConnectionOwned, u64) -> OnConnectionCloseFuture + Send + Sync>;

impl<'a> Server<'a> {
  pub fn new(config: &'a Config) -> Result<Self, Box<dyn std::error::Error>> {
    let server = Server {
      routes: Arc::new(Routes::new()),
      config,
      default_response_headers: Headers::new(),
      on_connection_close: None,
      request_id: 0,
    };

    Ok(server)
  }

  /// Starts listening on the supplied port. Also blocks the thread.
  pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    let server_address = format!("127.0.0.1:{}", self.config.port);

    let listener = TcpListener::bind(&server_address).await?;
    let mut connections = JoinSet::new();

    println!("Opened server on {server_address}",);

    loop {
      let (socket, _) = listener.accept().await?;

      let routes = self.routes.clone();
      let default_response_headers = self.default_response_headers.clone();
      let on_connection_close = self.on_connection_close.clone();
      let request_id = self.request_id;

      connections.spawn(async move {
        Self::handle_connection(
          socket,
          routes,
          &default_response_headers,
          on_connection_close,
          request_id,
        )
        .await
      });

      self.request_id += 1;
    }
  }

  pub fn set_default_response_headers(&mut self, headers: Headers) {
    self.default_response_headers = headers;
  }

  pub fn set_on_connection_close<F, Fut>(&mut self, callback: F)
  where
    F: Fn(HttpConnectionOwned, u64) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
  {
    self.on_connection_close = Some(Arc::new(move |stream, request_id| {
      Box::pin(callback(stream, request_id)) as OnConnectionCloseFuture
    }) as OnConnectionClose);
  }

  pub fn clear_on_connection_close(&mut self) {
    self.on_connection_close = None;
  }

  async fn handle_connection(
    stream: TcpStream,
    routes: Arc<Routes>,
    default_response_headers: &Headers,
    on_connection_close: Option<OnConnectionClose>,
    request_id: u64,
  ) {
    let mut http_connection = HttpConnectionOwned::new(stream);

    loop {
      let http_request =
        match Request::read_from_buffer(http_connection.get_reader()).await {
          Ok(request) => request,
          Err(error) => {
            eprintln!("Failed to read incoming TcpStream: {error}");
            break;
          }
        };

      let mut request_slugs: HashMap<String, String> = HashMap::new();
      let request_method = http_request.method.clone();
      let request_path = match Uri::percent_decode(&http_request.uri.path) {
        Ok(path) => path,
        Err(_) => {
          eprintln!("Failed to decode request path {}", http_request.uri.path);
          continue;
        }
      };

      let close_connection = match http_request.headers.get_header("Connection")
      {
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

      let request = ServerRequest {
        http_request,
        slugs: request_slugs,
        request_id,
      };

      let mut response = Response::new();
      match route {
        Some(route) => {
          response.headers = default_response_headers.clone();

          let result =
            (route.callback)(&request, &mut response, http_connection).await;

          match result {
            RouteResult::Respond(connection) => {
              http_connection = connection;

              if let Err(error) =
                http_connection.send_response(&mut response).await
              {
                eprintln!(
                  "Failed to serve {} route \"{}\": {error}",
                  request.http_request.method, request.http_request.uri.path
                );
              };
            }
            RouteResult::TakeOver => {
              return;
            }
          };
        }
        None => {
          println!(
            "Route not found. Method: {request_method}, Path: {request_path}",
          );

          response.status_code =
            http::http_status_codes::HttpStatusCode::Status404;
          if let Err(error) = http_connection.send_response(&mut response).await
          {
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
      callback(http_connection, request_id).await;
    };
  }
}
