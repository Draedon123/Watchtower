use crate::config::Config;
use http::{method::HTTPMethod, request::Request, response::Response};
use std::{collections::HashMap, net::TcpListener};

pub enum URLSection {
  Literal(String),
  Slug(String),
}

struct Route {
  method: HTTPMethod,
  path: Vec<URLSection>,
  callback: Box<dyn Fn(&mut ServerRequest, &mut Response) -> ()>,
}

impl Route {
  fn path_from_url(url: &str) -> Vec<URLSection> {
    let split_url: Vec<_> = url.split("/").collect();
    let mut path: Vec<URLSection> = Vec::with_capacity(split_url.len());

    split_url.iter().for_each(|section| {
      if section.starts_with("[") && section.ends_with("]") {
        let slug_name = (&section[1..(section.len() - 1)]).to_string();
        path.push(URLSection::Slug(slug_name));
      } else {
        path.push(URLSection::Literal(section.to_string()));
      };
    });

    path
  }

  fn matches(&self, url: &str) -> Option<HashMap<String, String>> {
    let split_url: Vec<_> = url.split("/").collect();

    if split_url.len() != self.path.len() {
      None
    } else {
      let mut slugs: HashMap<String, String> = HashMap::new();

      for i in 0..split_url.len() {
        match &self.path[i] {
          URLSection::Literal(section) => {
            if section != split_url[i] {
              return None;
            }
          }
          URLSection::Slug(slug) => {
            slugs.insert(slug.clone(), split_url[i].to_string());
          }
        }
      }

      Some(slugs)
    }
  }
}

pub struct ServerRequest<'a> {
  pub http_request: &'a mut Request,
  pub slugs: HashMap<String, String>,
}

impl<'a> ServerRequest<'a> {
  pub fn respond(&mut self, response: &Response) {
    self.http_request.respond(response)
  }
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

      let mut http_request = Request::read_from_stream(stream);

      let mut request_slugs: HashMap<String, String> = HashMap::new();
      let request_method = http_request.method.clone();
      let request_path = http_request.path.clone();

      println!("Received {} request for {}", request_method, request_path);

      let route = self.routes.iter().find(|route| {
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

          (route.callback)(&mut request, &mut response);
        }
        None => {
          println!(
            "Route not found. Method: {}, Path: {}",
            request_method, request_path
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
    F: Fn(&mut ServerRequest, &mut Response) -> () + 'static,
  {
    self.routes.push(Route {
      path: Route::path_from_url(path),
      method: method,
      callback: Box::new(on_request),
    })
  }
}
