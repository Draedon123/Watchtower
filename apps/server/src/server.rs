use crate::{config::Config, route::Route, thread_pool::ThreadPool};
use http::{header::Headers, method::HTTPMethod, request::Request, response::Response};
use std::{
  collections::HashMap,
  io::BufReader,
  net::{TcpListener, TcpStream},
  sync::{Arc, RwLock},
};

pub struct ServerRequest<'a> {
  pub http_request: &'a mut Request,
  pub slugs: HashMap<String, String>,
}

impl<'a> ServerRequest<'a> {
  pub fn respond(&mut self, response: &Response) -> Result<(), String> {
    self.http_request.respond(response)
  }
}

pub struct Server<'a> {
  pub default_response_headers: Arc<RwLock<Headers>>,

  routes: Arc<RwLock<Vec<Route>>>,
  config: &'a Config,
  thread_pool: ThreadPool,
}

impl<'a> Server<'a> {
  pub fn new(config: &'a Config) -> Result<Server<'a>, Box<dyn std::error::Error>> {
    let server = Server {
      routes: Arc::new(RwLock::new(Vec::new())),
      config,
      thread_pool: ThreadPool::new(4),
      default_response_headers: Arc::new(RwLock::new(Headers::new())),
    };

    Ok(server)
  }

  /// Starts listening on the supplied port. Also blocks the thread.
  pub fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
    let server_address = format!("127.0.0.1:{}", self.config.port);

    let listener = TcpListener::bind(&server_address)?;

    println!("Opened server on {server_address}");

    for stream in listener.incoming() {
      let stream = if let Ok(stream) = stream {
        stream
      } else {
        eprintln!("Failed to read incoming TcpStream");
        continue;
      };

      let routes = self.routes.clone();
      let default_response_headers = self.default_response_headers.clone();

      self.thread_pool.execute(move || {
        Self::handle_connection(stream, routes, default_response_headers);
      });
    }

    Ok(())
  }

  pub fn route<F>(&mut self, path: &str, method: HTTPMethod, on_request: F)
  where
    F: Fn(&mut ServerRequest, &mut Response) + Send + Sync + 'static,
  {
    self.routes.write().unwrap().push(Route {
      path: Route::path_from_url(path),
      method: method,
      callback: Arc::new(on_request),
    })
  }

  pub fn set_default_response_headers(&mut self, headers: Headers) {
    self.default_response_headers = Arc::new(RwLock::new(headers));
  }

  fn handle_connection(
    stream: TcpStream,
    routes: Arc<RwLock<Vec<Route>>>,
    default_response_headers: Arc<RwLock<Headers>>,
  ) {
    let mut reader = BufReader::new(&stream);
    let routes = routes.read().unwrap();

    loop {
      let mut http_request = match Request::read_from_buffer(&stream, &mut reader) {
        Ok(Some(request)) => request,
        Ok(None) => break,
        Err(error) => {
          eprintln!("Failed to read incoming TcpStream: {error}");
          continue;
        }
      };

      let mut request_slugs: HashMap<String, String> = HashMap::new();
      let request_method = http_request.method.clone();
      let request_path = http_request.path.clone();

      let close_connection = match http_request.headers.get_header("Connection") {
        Some(value) => value == "close",
        None => false,
      };

      println!("Received {request_method} request for {request_path}");

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

      match route {
        Some(route) => {
          let mut response = Response::new();

          response.headers = default_response_headers.read().unwrap().clone();

          (route.callback)(&mut request, &mut response);
        }
        None => {
          println!("Route not found. Method: {request_method}, Path: {request_path}",);
          let mut response = Response::new();

          response.status_code = http::status_codes::StatusCode::Status404;
          if let Err(error) = request.respond(&response) {
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
  }
}
