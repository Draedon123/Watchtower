use config::Config;
use server::Server;
use std::env;
use std::sync::{Arc, RwLock};

mod config;
mod data;
mod route;
mod server;
mod thread_pool;

use http::method::HTTPMethod;
use http::status_codes::StatusCode;

use crate::data::{Data, DataType};

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let args: Vec<String> = env::args().collect();
  let config = Config::new(&args);

  let mut server = Server::new(&config)?;
  let data = Arc::new(RwLock::new(Data::new(10)));

  server.route("/", HTTPMethod::Get(), |request, response| {
    response.status_code = StatusCode::Status200;
    if let Err(error) = request.respond(response) {
      eprintln!("Failed to serve route \"/\": {error}");
    };
  });

  let data_reader_1 = data.clone();
  server.route(
    "/get/[machine]/[key]",
    HTTPMethod::Post(),
    move |request, response| {
      response
        .headers
        .set_header("Access-Control-Allow-Origin", "*");

      let machine = request.slugs.get("machine").unwrap().clone();
      let key = request.slugs.get("key").unwrap().clone();

      let data = data_reader_1.read().unwrap();
      let recent_data = data.get_machine_data_recent(&machine, &key);

      match recent_data {
        Ok(data_entry) => {
          response.set_content(&data_entry.data.to_string());
          response.status_code = StatusCode::Status200;
        }
        Err(error) => {
          response.status_code = StatusCode::Status400;
          response.set_content(&error.to_string());
        }
      };

      if let Err(error) = request.respond(response) {
        eprintln!("Failed to serve POST route \"/get/{machine}/{key}\": {error}",);
      };
    },
  );

  let data_reader_2 = data.clone();
  server.route(
    "/get/machines",
    HTTPMethod::Post(),
    move |request, response| {
      response
        .headers
        .set_header("Access-Control-Allow-Origin", "*");

      let data = data_reader_2.read().unwrap();

      response.status_code = StatusCode::Status200;
      response.set_content(
        &data
          .list_machines()
          .iter()
          .map(|string| (*string).clone())
          .collect::<Vec<_>>()
          .join("\n"),
      );

      if let Err(error) = request.respond(response) {
        eprintln!("Failed to serve POST route \"/get/machines\": {error}");
      };
    },
  );

  let data_writer_1 = data.clone();
  let server_password_1 = config.password.clone();
  server.route(
    "/set/[machine]",
    HTTPMethod::Post(),
    move |request, response| {
      response
        .headers
        .set_header("Access-Control-Allow-Origin", "*");

      let machine = request.slugs.get("machine").unwrap().clone();
      let password = match request.http_request.headers.get_header("Authorization") {
        Some(password) => password.clone(),
        None => String::new(),
      };
      if password != server_password_1 {
        println!("Received unauthenticated request");

        response.status_code = StatusCode::Status401;
        if let Err(error) = request.respond(response) {
          eprintln!("Failed to serve POST route \"/set/{machine}\": {error}");
        };
        return;
      }

      let mut data = data_writer_1.write().unwrap();

      response.status_code = if data.register_machine(&machine) {
        StatusCode::Status200
      } else {
        StatusCode::Status400
      };

      if let Err(error) = request.respond(response) {
        eprintln!("Failed to serve POST route \"/set/{machine}\": {error}",);
      };
    },
  );

  let data_writer_2 = data.clone();
  let server_password_2 = config.password.clone();
  server.route(
    "/set/[machine]/[key]",
    HTTPMethod::Post(),
    move |request, response| {
      response
        .headers
        .set_header("Access-Control-Allow-Origin", "*");

      let content: Vec<_> = request.http_request.get_content().lines().collect();
      let password = match request.http_request.headers.get_header("Authorization") {
        Some(password) => password.clone(),
        None => String::new(),
      };

      let machine = request.slugs.get("machine").unwrap().clone();
      let key = request.slugs.get("key").unwrap().clone();

      if password != server_password_2 {
        println!("Received unauthenticated request");

        response.status_code = StatusCode::Status401;
        if let Err(error) = request.respond(response) {
          eprintln!("Failed to serve POST route \"/set/{machine}/{key}\": {error}",);
        };
        return;
      }

      let value = if content.is_empty() { "" } else { content[0] };

      let mut data = data_writer_2.write().unwrap();

      match value.parse::<f32>() {
        Ok(value) => {
          response.status_code = StatusCode::Status200;

          if let Err(error) = data.set_data(&machine, &key, DataType::F32Value(value)) {
            eprintln!("Failed to set \"{key}\"=\"{value}\" for machine \"{machine}\": {error}");
          };
        }
        Err(_) => {
          response.status_code = StatusCode::Status400;
          response.set_content("Invalid value");
        }
      };

      if let Err(error) = request.respond(response) {
        eprintln!("Failed to serve POST route \"/set/{machine}/{key}\": {error}",);
      };
    },
  );

  server.start().unwrap();

  Ok(())
}
