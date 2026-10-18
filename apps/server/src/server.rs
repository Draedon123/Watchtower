use crate::config::Config;
use http::{method::HTTPMethod, request::Request, response::Response};
use std::net::TcpListener;

struct Route {
  method: HTTPMethod,
  path: String,
  callback: Box<dyn Fn(Request, Response) -> ()>,
}

pub struct Server<'a> {
  routes: Vec<Route>,
  config: &'a Config,
}

impl<'a> Server<'a> {
  pub fn new(config: &'a Config) -> Result<Server<'a>, Box<dyn std::error::Error>> {
    let server = Server {
      routes: Vec::new(),
      config,
    };

    Ok(server)
  }

  /// Starts listening on the supplied port. Also blocks the thread.
  pub fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
    let server_address = format!("127.0.0.1:{}", self.config.port);

    let listener = TcpListener::bind(&server_address)?;

    println!("Opened server on {server_address}");

    for stream in listener.incoming() {
      let stream = stream.unwrap();

      let mut request = Request::new(stream);

      match self
        .routes
        .iter()
        .find(|route| route.method == request.method && route.path == request.path)
      {
        Some(route) => {
          let response = Response::new();

          (route.callback)(request, response);
        }
        None => {
          println!(
            "Route not found. Method: {}, Path: {}",
            request.method, request.path
          );
          let mut response = Response::new();

          response.status_code = http::status_codes::StatusCode::Status404;
          request.respond(&response);
        }
      };
    }

    Ok(())
  }

  pub fn route<F>(&mut self, path: &str, method: HTTPMethod, on_request: F)
  where
    F: Fn(Request, Response) -> () + 'static,
  {
    self.routes.push(Route {
      path: path.to_string(),
      method: method,
      callback: Box::new(on_request),
    })
  }
}
