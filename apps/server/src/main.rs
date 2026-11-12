use http::header::Headers;
use http::response::Response;
use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;

use http::method::HTTPMethod;
use http::status_codes::StatusCode;

use watchtower_server::{
  config::{Config, RunMode},
  data::{Data, DataType},
  help::display_help_message,
  route::Routes,
  server::{Server, ServerRequest},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let args: Vec<String> = env::args().collect();
  let config = Config::new(&args);

  if let RunMode::Help = config.mode {
    display_help_message();

    return Ok(());
  };

  let data = Arc::new(RwLock::new(Data::new(10)));
  let mut server = Server::new(&config)?;
  server.routes = Arc::new(get_routes(&config, &data));

  let default_response_headers = Headers::from(vec![("Access-Control-Allow-Origin", "*")]);
  server.set_default_response_headers(default_response_headers);

  let data_writer = data.clone();
  server.set_on_connection_close(move |stream| {
    let data = data_writer.clone();

    async move {
      let ip = stream.peer_addr().unwrap().to_string();

      let machine = data.write().await.unregister_machine(&ip);

      match machine {
        Some(machine) => println!("Successfully unregistered {machine} ({ip})"),
        None => println!("Failed to unregister machine from ip: {ip}"),
      };
    }
  });

  server.start().await?;

  Ok(())
}

fn authenticate(request: &mut ServerRequest<'_>, server_password: &str) -> bool {
  let submitted_password = match request.http_request.headers.get_header("Authorization") {
    Some(password) => password,
    None => &String::new(),
  };

  submitted_password == server_password
}

fn respond_if_unauthenticated(
  request: &mut ServerRequest<'_>,
  response: &mut Response,
  server_password: &str,
) -> bool {
  let authenticated = authenticate(request, server_password);

  if !authenticated {
    println!("Received unauthenticated request");
    response.status_code = StatusCode::Status401;
  }

  authenticated
}

fn get_routes(config: &Config, data: &Arc<RwLock<Data>>) -> Routes {
  let mut routes = Routes::new();

  let password = Arc::new(config.password.clone());

  routes.route("/", HTTPMethod::Get(), |_, mut response| {
    Box::pin(async move {
      response.status_code = StatusCode::Status200;

      response
    })
  });

  let data_reader = data.clone();
  routes.route(
    "/get/[machine]/[key]",
    HTTPMethod::Post(),
    move |request, mut response| {
      let data = data_reader.clone();

      Box::pin(async move {
        let machine = request.slugs.get("machine").unwrap();
        let key = request.slugs.get("key").unwrap();

        let data = data.read().await;
        let recent_data = data.get_machine_data_recent(machine, key);
        match recent_data {
          Ok(data_entry) => {
            response.set_content(&data_entry.to_string());
            response.status_code = StatusCode::Status200;
          }
          Err(error) => {
            response.set_content(&error.to_string());
            response.status_code = StatusCode::Status400;
          }
        };

        response
      })
    },
  );

  let data_reader = data.clone();
  routes.route(
    "/all/[machine]",
    HTTPMethod::Post(),
    move |request, mut response| {
      let data = data_reader.clone();

      Box::pin(async move {
        let machine = request.slugs.get("machine").unwrap();

        let data = data.read().await;
        let machine_data = data.get_machine_data_all(machine);
        match machine_data {
          Ok(data_entry) => {
            response.set_content(
              &data_entry
                .keys()
                .filter_map(|key| {
                  data
                    .get_machine_data_recent(machine, key)
                    .ok()
                    .map(|data| (key, data))
                })
                .fold(String::new(), |joined, (key, data)| {
                  format!("{joined}\n{key}\n{data}")
                })
                .trim(),
            );
            response.status_code = StatusCode::Status200;
          }
          Err(error) => {
            response.set_content(&error.to_string());
            response.status_code = StatusCode::Status400;
          }
        };

        response
      })
    },
  );

  let data_reader = data.clone();
  routes.route(
    "/get/machines",
    HTTPMethod::Post(),
    move |_, mut response| {
      let data = data_reader.clone();

      Box::pin(async move {
        {
          response.status_code = StatusCode::Status200;
          response.set_content(
            &data
              .read()
              .await
              .list_machines()
              .iter()
              .map(|string| string.as_str())
              .collect::<Vec<_>>()
              .join("\n"),
          );

          response
        }
      })
    },
  );

  let data_writer = data.clone();
  let server_password = password.clone();
  routes.route(
    "/set/[machine]",
    HTTPMethod::Post(),
    move |request, mut response| {
      let data = data_writer.clone();
      let server_password = Arc::clone(&server_password);

      Box::pin(async move {
        {
          if !respond_if_unauthenticated(request, &mut response, &server_password) {
            return response;
          };

          let machine = request.slugs.get("machine").unwrap();

          response.status_code = if data.write().await.register_machine(
            request.http_request.stream.peer_addr().unwrap().to_string(),
            machine,
          ) {
            StatusCode::Status200
          } else {
            StatusCode::Status400
          };

          response
        }
      })
    },
  );

  let data_writer = data.clone();
  let server_password = password.clone();
  routes.route(
    "/set/[machine]/[key]",
    HTTPMethod::Post(),
    move |request, mut response| {
      let data = data_writer.clone();
      let server_password = Arc::clone(&server_password);

      Box::pin(async move {
      if !respond_if_unauthenticated(request, &mut response, &server_password) {
        return response;
      };

      let content: Vec<_> = request.http_request.get_content().lines().collect();
      let machine = request.slugs.get("machine").unwrap();
      let key = request.slugs.get("key").unwrap();

      let value_type = if content.is_empty() { "" } else { content[0] };
      let value = if content.is_empty() { "" } else { content[1] };
      let new_data = DataType::from((value_type, value));

      match new_data {
        DataType::F32Value(value) =>{
          response.status_code = StatusCode::Status200;

          if let Err(error) = data.write().await.set_data(machine, key, DataType::F32Value(value)) {
            eprintln!(
              "Failed to set f32 value of \"{key}\"=\"{value}\" for machine \"{machine}\": {error}"
            );
          };
        }
        DataType::Vec2U64Value(value) =>{
          response.status_code = StatusCode::Status200;

          if let Err(error) = data.write().await.set_data(machine, key, DataType::Vec2U64Value(value)) {
            eprintln!(
              "Failed to set vec2u64 value of \"{key}\"=\"({}, {})\" for machine \"{machine}\": {error}", value.0, value.1
            );
          };
        }
        DataType::Unknown(value) => {
          response.status_code = StatusCode::Status400;
          response.set_content(&format!("Invalid value \"{value}\""));
        }
      };

response    })},
  );

  routes
}
