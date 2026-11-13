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

  let data = Arc::new(RwLock::new(Data::new()));
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
    "/machine/[machine]/get",
    HTTPMethod::Post(),
    move |request, mut response| {
      let data = data_reader.clone();

      Box::pin(async move {
        let data = data.read().await;

        let machine = request.slugs.get("machine").unwrap();
        // TODO: optimise
        let keys = if request.http_request.get_content().is_empty() {
          data.list_machine_keys(machine).unwrap()
        } else {
          request.http_request.get_content().lines().collect()
        };

        let mut serialised_data = String::new();

        for key in keys {
          let recent_data = data.get_machine_data(machine, key);

          match recent_data {
            Ok(data_entry) => {
              serialised_data.push_str(&format!("{key}\n{data_entry}\n"));
            }
            Err(error) => {
              eprintln!("Failed to fetch \"{key}\" for \"{machine}\": {error}");
            }
          };
        }

        response.status_code = StatusCode::Status200;
        response.set_content(&serialised_data);
        response
      })
    },
  );

  let data_reader = data.clone();
  routes.route(
    "/machine/list",
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
    "/machine/register/[machine]",
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
  routes.route("/machine/[machine]/set", HTTPMethod::Post(), move |request, mut response| {
    let data = data_writer.clone();
    let server_password = Arc::clone(&server_password);

    Box::pin(async move {
      if !respond_if_unauthenticated(request, &mut response, &server_password) {
        return response;
      };

      let content: Vec<_> = request.http_request.get_content().lines().collect();
      let machine = request.slugs.get("machine").unwrap();

      response.status_code = StatusCode::Status200;

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

      response
    })
  });

  let data_reader = data.clone();
  routes.route(
    "/stats/[key]",
    HTTPMethod::Post(),
    move |request, mut response| {
      let data = data_reader.clone();

      Box::pin(async move {
        let data = data.read().await;

        let key = request.slugs.get("key").unwrap();
        // TODO: optimise
        let machines: Vec<_> = if request.http_request.get_content().is_empty() {
          data
            .list_machines()
            .iter()
            .map(|machine| &machine[..])
            .collect()
        } else {
          request.http_request.get_content().lines().collect()
        };

        let mut serialised_data = String::new();

        for machine in machines {
          let recent_data = data.get_machine_data(machine, key);

          match recent_data {
            Ok(data_entry) => {
              serialised_data.push_str(&format!("{machine}\n{data_entry}\n"));
            }
            Err(error) => {
              eprintln!("Failed to fetch \"{key}\" for \"{machine}\": {error}");
            }
          };
        }

        response.status_code = StatusCode::Status200;
        response.set_content(&serialised_data);
        response
      })
    },
  );

  routes
}
