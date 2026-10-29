use http::header::Headers;
use http::response::Response;
use std::env;
use std::sync::{Arc, RwLock};

use http::method::HTTPMethod;
use http::status_codes::StatusCode;

use watchtower_server::{
  config::Config,
  data::{Data, DataType},
  server::{Server, ServerRequest},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let args: Vec<String> = env::args().collect();
  let config = Config::new(&args);

  let mut server = Server::new(&config)?;
  let data = Arc::new(RwLock::new(Data::new(10)));

  let mut default_response_headers = Headers::new();
  default_response_headers.set_header("Access-Control-Allow-Origin", "*");
  server.set_default_response_headers(default_response_headers);

  let data_writer_1 = data.clone();
  server.set_on_connection_close(move |stream| {
    let ip = stream.peer_addr().unwrap().to_string();

    let machine = data_writer_1.write().unwrap().unregister_machine(&ip);

    match machine {
      Some(machine) => println!("Successfully unregistered {machine} ({ip})"),
      None => println!("Failed to unregister machine from ip: {ip}"),
    }
  });

  let server_password = Arc::new(config.password.clone());

  server.route("/", HTTPMethod::Get(), |request, response, _| {
    response.status_code = StatusCode::Status200;

    try_respond(request, response);
  });

  let data_reader_1 = data.clone();
  server.route(
    "/get/[machine]/[key]",
    HTTPMethod::Post(),
    move |request, response, _| {
      let machine = request.slugs.get("machine").unwrap();
      let key = request.slugs.get("key").unwrap();

      let data = data_reader_1.read().unwrap();
      let recent_data = data.get_machine_data_recent(machine, key);

      match recent_data {
        Ok(data_entry) => {
          response.set_content(&data_entry.to_string());
          response.status_code = StatusCode::Status200;
        }
        Err(error) => {
          response.status_code = StatusCode::Status400;
          response.set_content(&error.to_string());
        }
      };

      try_respond(request, response);
    },
  );

  let data_reader_2 = data.clone();
  server.route(
    "/get/machines",
    HTTPMethod::Post(),
    move |request, response, _| {
      let data = data_reader_2.read().unwrap();

      response.status_code = StatusCode::Status200;
      response.set_content(
        &data
          .list_machines()
          .iter()
          .map(|string| string.as_str())
          .collect::<Vec<_>>()
          .join("\n"),
      );

      try_respond(request, response);
    },
  );

  let data_writer_2 = data.clone();
  let server_password_1 = server_password.clone();
  server.route(
    "/set/[machine]",
    HTTPMethod::Post(),
    move |request, response, stream| {
      if !is_authenticated(request, response, &server_password_1) {
        return;
      };

      let machine = request.slugs.get("machine").unwrap();
      let mut data = match data_writer_2.write() {
        Ok(data) => data,
        Err(error) => {
          response.status_code = StatusCode::Status500;
          response.set_content(&error.to_string());

          try_respond(request, response);
          return;
        }
      };

      response.status_code =
        if data.register_machine(stream.peer_addr().unwrap().to_string(), machine) {
          StatusCode::Status200
        } else {
          StatusCode::Status400
        };

      try_respond(request, response);
    },
  );

  let data_writer_3 = data.clone();
  let server_password_2 = server_password.clone();
  server.route(
    "/set/[machine]/[key]",
    HTTPMethod::Post(),
    move |request, response, _| {
      if !is_authenticated(request, response, &server_password_2) {
        return;
      };

      let content: Vec<_> = request.http_request.get_content().lines().collect();
      let machine = request.slugs.get("machine").unwrap();
      let key = request.slugs.get("key").unwrap();

      let value_type = if content.is_empty() { "" } else { content[0] };
      let value = if content.is_empty() { "" } else { content[1] };
      let data = DataType::from((value_type, value));

      let mut all_data = match data_writer_3.write() {
        Ok(data) => data,
        Err(error) => {
          response.status_code = StatusCode::Status500;
          response.set_content(&error.to_string());

          try_respond(request, response);
          return;
        }
      };

      match data {
        DataType::F32Value(value) =>{
          response.status_code = StatusCode::Status200;

          if let Err(error) = all_data.set_data(machine, key, DataType::F32Value(value)) {
            eprintln!(
              "Failed to set f32 value of \"{key}\"=\"{value}\" for machine \"{machine}\": {error}"
            );
          };
        }
        DataType::Vec2U64Value(value) =>{
          response.status_code = StatusCode::Status200;

          if let Err(error) = all_data.set_data(machine, key, DataType::Vec2U64Value(value)) {
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

      try_respond(request, response);
    },
  );

  server.start().unwrap();

  Ok(())
}

fn is_authenticated(
  request: &mut ServerRequest,
  response: &mut Response,
  server_password: &String,
) -> bool {
  let submitted_password = match request.http_request.headers.get_header("Authorization") {
    Some(password) => password,
    None => &String::new(),
  };

  if *submitted_password != *server_password {
    println!("Received unauthenticated request");

    response.status_code = StatusCode::Status401;
    if let Err(error) = request.respond(response) {
      eprintln!(
        "Failed to serve {} route \"{}\": {error}",
        request.http_request.method, request.http_request.path
      );
    };

    false
  } else {
    true
  }
}

fn try_respond(request: &mut ServerRequest, response: &mut Response) {
  if let Err(error) = request.respond(response) {
    eprintln!(
      "Failed to serve {} route \"{}\": {error}",
      request.http_request.method, request.http_request.path
    );
  };
}
