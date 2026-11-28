use http::response::Response;
use http::{body::Body, header::Headers};
use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinSet;
use watchtower_server::route::RouteResult;
use websocket::server::Server as WsServer;
use websocket::server_client::EventType;
use websocket::ws_server_handle::WsServerHandle;

use http::http_method::HttpMethod;
use http::http_status_codes::HttpStatusCode;

use watchtower_server::{
  config::{Config, RunMode},
  data::{Data, DataType},
  help::display_help_message,
  route::Routes,
  server::{Server as HttpServer, ServerRequest},
};

#[tokio::main]
async fn main() {
  let args: Vec<String> = env::args().collect();
  let config = Config::new(&args);

  if let RunMode::Help = config.mode {
    display_help_message();

    return;
  };

  let data = Arc::new(RwLock::new(Data::new()));

  let mut http_server = HttpServer::new(config.port).unwrap();
  let mut websocket_server = WsServer::new();
  let ws_server_handle = Arc::new(websocket_server.get_handle());

  http_server.routes =
    Arc::new(get_routes(&config, &data, ws_server_handle.clone()));

  let default_response_headers =
    Headers::from(vec![("Access-Control-Allow-Origin", "*")]);
  http_server.set_default_response_headers(default_response_headers);

  let data_writer = data.clone();
  http_server.set_on_connection_close(move |_, request_id| {
    let data = data_writer.clone();

    async move {
      let machine = data.write().await.unregister_machine(request_id);

      if let Some(machine) = machine {
        println!("Successfully unregistered {machine} (id: {request_id})")
      }
    }
  });

  let mut joinset = JoinSet::<()>::new();

  joinset.spawn(async move {
    http_server.start().await.unwrap();
  });

  joinset.spawn(async move {
    websocket_server.start().await;
  });

  joinset.join_all().await;
}

fn authenticate(request: &ServerRequest, server_password: &str) -> bool {
  let submitted_password =
    match request.http_request.headers.get_header("Authorization") {
      Some(password) => password,
      None => &String::new(),
    };

  submitted_password == server_password
}

fn respond_if_unauthenticated(
  request: &ServerRequest,
  response: &mut Response,
  server_password: &str,
) -> bool {
  let authenticated = authenticate(request, server_password);

  if !authenticated {
    println!("Received unauthenticated request");
    response.status_code = HttpStatusCode::Status401;
  }

  authenticated
}

fn get_routes(
  config: &Config,
  data: &Arc<RwLock<Data>>,
  handle: Arc<WsServerHandle>,
) -> Routes {
  let mut routes = Routes::new();

  let password = Arc::new(config.password.clone());

  routes.route("/", HttpMethod::Get, |_, response, connection| {
    Box::pin(async move {
      response.status_code = HttpStatusCode::Status200;
      response.body = Body::Text(String::from("ok"));

      RouteResult::Respond(connection)
    })
  });

  let data_reader = data.clone();
  routes.route(
    "/machine/[machine]/get",
    HttpMethod::Post,
    move |request, response, connection| {
      let data = data_reader.clone();

      Box::pin(async move {
        let machine = request
          .slugs
          .get("machine")
          .expect("[machine] slug is present in route path");
        // TODO: optimise
        let body = match request.http_request.body.utf8() {
          Ok(body) => body,
          Err(_) => {
            invalid_body_error(response);

            return RouteResult::Respond(connection);
          }
        };

        let data = data.read().await;
        let keys = if body.is_empty() {
          match data.list_machine_keys(machine) {
            Ok(data) => data,
            Err(error) => {
              response.status_code = error.get_http_error_code();
              response.body = Body::from(error.to_string());
              return RouteResult::Respond(connection);
            }
          }
        } else {
          body.lines().collect()
        };

        let mut serialised_data = String::new();

        for key in keys {
          let recent_data = data.get_machine_data(machine, key);

          match recent_data {
            Ok(data_entry) => {
              serialised_data
                .push_str(&format!("{key}\n{}\n", data_entry.serialise()));
            }
            Err(error) => {
              eprintln!("Failed to fetch \"{key}\" for \"{machine}\": {error}");
            }
          };
        }

        response.status_code = HttpStatusCode::Status200;
        response.body = Body::from(serialised_data);

        RouteResult::Respond(connection)
      })
    },
  );

  let data_reader = data.clone();
  routes.route(
    "/machine/list",
    HttpMethod::Post,
    move |_, response, connection| {
      let data = data_reader.clone();

      Box::pin(async move {
        {
          response.status_code = HttpStatusCode::Status200;
          response.body = Body::from(
            data
              .read()
              .await
              .list_machines()
              .iter()
              .map(|string| string.as_str())
              .collect::<Vec<_>>()
              .join("\n"),
          );

          RouteResult::Respond(connection)
        }
      })
    },
  );

  let data_writer = data.clone();
  let server_password = password.clone();
  routes.route(
    "/machine/register/[machine]",
    HttpMethod::Post,
    move |request, response, connection| {
      let data = data_writer.clone();
      let server_password = Arc::clone(&server_password);

      Box::pin(async move {
        {
          if !respond_if_unauthenticated(request, response, &server_password) {
            return RouteResult::Respond(connection);
          };

          let machine = request
            .slugs
            .get("machine")
            .expect("[machine] slug is present in route path");

          response.status_code = if data
            .write()
            .await
            .register_machine(request.request_id, machine)
          {
            HttpStatusCode::Status200
          } else {
            HttpStatusCode::Status400
          };

          RouteResult::Respond(connection)
        }
      })
    },
  );

  let data_writer = data.clone();
  let server_password = password.clone();
  routes.route("/machine/[machine]/set", HttpMethod::Post, move |request, response, connection| {
    let data = data_writer.clone();
    let server_password = Arc::clone(&server_password);

    Box::pin(async move {
      if !respond_if_unauthenticated(request, response, &server_password) {
        return RouteResult::Respond(connection);
      };

      let body = match request.http_request.body.utf8() {
        Ok(body) => body,
        Err(_) => {
          invalid_body_error(response);

          return RouteResult::Respond(connection);
        }
      };
      let content: Vec<_> = body.lines().collect();
      let machine = request.slugs.get("machine").expect("[machine] slug is present in route path");

      response.status_code = HttpStatusCode::Status200;

      let mut data = data.write().await;
      for i in 0..content.len() / 4 {
        let key = content[i * 4];
        let value_type = content[i * 4 + 1];
        let value = content[i * 4 + 2];
        // let timestamp = content[1 * 4 + 3];

        let new_data = DataType::from((value_type, value));
        if let DataType::Unknown(value) = new_data {
          eprintln!("Invalid value \"{value}\" (unknown type: {value_type})");
        } else {
          if let Err(error) = data.set_data(machine, key, new_data) {
            eprintln!("Failed to set value of {value_type} \"{key}\"=\"{value}\" for machine \"{machine}\": {error}");
          };
        }
      }

      RouteResult::Respond(connection)
    })
  });

  let data_reader = data.clone();
  routes.route(
    "/stats/[key]",
    HttpMethod::Post,
    move |request, response, connection| {
      let data = data_reader.clone();

      Box::pin(async move {
        let data = data.read().await;

        let key = request
          .slugs
          .get("key")
          .expect("[key] slug is present in route path");
        let body = match request.http_request.body.utf8() {
          Ok(body) => body,
          Err(_) => {
            invalid_body_error(response);
            return RouteResult::Respond(connection);
          }
        };
        // TODO: optimise
        let machines: Vec<_> = if body.is_empty() {
          data
            .list_machines()
            .iter()
            .map(|machine| &machine[..])
            .collect()
        } else {
          body.lines().collect()
        };

        let mut serialised_data = String::new();

        for machine in machines {
          let recent_data = data.get_machine_data(machine, key);

          match recent_data {
            Ok(data_entry) => {
              serialised_data
                .push_str(&format!("{machine}\n{}\n", data_entry.serialise()));
            }
            Err(error) => {
              eprintln!("Failed to fetch \"{key}\" for \"{machine}\": {error}");
            }
          };
        }

        response.status_code = HttpStatusCode::Status200;
        response.body = Body::from(serialised_data);

        RouteResult::Respond(connection)
      })
    },
  );

  let handle = handle.clone();
  routes.route("/machine/[machine]/websocket", HttpMethod::Get, move |request, response, connection| {
    let server_handle = handle.clone();

    Box::pin(async move {
      let machine = request.slugs.get("machine").expect("[machine] slug is present in route path");
      let result = server_handle.handle_http_request(machine.clone(), &request.http_request, connection).await;

      match result {
        Ok(_) => {
          let machine_handle = server_handle.get_event_receiver(machine.clone()).await;

          match server_handle.send_text(machine.clone(), String::from("Hello from http server")).await {
            Ok(reply) => println!("HTTP Server received reply from WebSocket Server: {reply}"),
            Err(error) => eprintln!("HTTP Server received no reply from WebSocket Server: {error}"),
          };

          let machine = machine.clone();

          if let Some(mut handle) = machine_handle {
            tokio::spawn(async move {
              while let Some(event) = handle.recv().await {
                match event.event {
                  EventType::Close(_,_) => {
                    break;
                  },
                  EventType::Text(message) => {
                    let _ = server_handle.send_text(machine.clone(), format!("Server received: \"{message}\"")).await;
                  },
                  _ => {}
                }
              }
            });
          };
        }
        Err((error, mut connection)) => {
          response.status_code = HttpStatusCode::Status400;
          response.headers.set_header("Connection", "close");
          response.body = Body::Text(error);

          if let Err(error) = connection.as_mut().send_response(response).await {
            eprintln!("Failed to send response for failed WebSocket connection: {error}");
          };
        }
      }

      RouteResult::TakeOver
    })
  });

  routes
}

fn invalid_body_error(response: &mut Response) {
  response.status_code = HttpStatusCode::Status400;
}
