use config::Config;
use server::Server;
use std::env;
use std::sync::{Arc, RwLock};

mod config;
mod data;
mod server;

use http::method::HTTPMethod;
use http::status_codes::StatusCode;

use crate::data::{Data, DataType};

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let args: Vec<String> = env::args().collect();
  let config = Config::new(&args);

  let mut server = Server::new(&config).unwrap();
  let data = Arc::new(RwLock::new(Data::new(10)));

  server.route("/", HTTPMethod::Get(), |request, response| {
    response.status_code = StatusCode::Status200;
    request.respond(response);
  });

  let data_reader_1 = data.clone();
  server.route(
    "/get/[machine]/[key]",
    HTTPMethod::Post(),
    move |request, response| {
      response
        .headers
        .set_header("Access-Control-Allow-Origin", "*");

      let machine = request.slugs.get("machine").unwrap();
      let key = request.slugs.get("key").unwrap();

      let data = data_reader_1.read().unwrap();
      let recent_data = data.get_machine_data_recent(machine, key);

      match recent_data {
        Ok(data_entry) => {
          response.set_content(&data_entry.data.to_string());
          response.status_code = StatusCode::Status200;
        }
        Err(error) => {
          response.status_code = StatusCode::Status400;
          response.set_content(&format!("{}", error));
        }
      };

      request.respond(response);
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

      request.respond(response);
    },
  );

  let data_writer_1 = data.clone();
  server.route(
    "/set/[machine]",
    HTTPMethod::Post(),
    move |request, response| {
      response
        .headers
        .set_header("Access-Control-Allow-Origin", "*");

      let machine = request.slugs.get("machine").unwrap();

      let mut data = data_writer_1.write().unwrap();

      response.status_code = if data.register_machine(machine) {
        StatusCode::Status200
      } else {
        StatusCode::Status400
      };

      request.respond(response);
    },
  );

  let data_writer_2 = data.clone();
  server.route(
    "/set/[machine]/[key]",
    HTTPMethod::Post(),
    move |request, response| {
      response
        .headers
        .set_header("Access-Control-Allow-Origin", "*");

      let machine = request.slugs.get("machine").unwrap();
      let key = request.slugs.get("key").unwrap();
      let value = request.http_request.get_content();

      let mut data = data_writer_2.write().unwrap();

      match value.parse::<f32>() {
        Ok(value) => {
          response.status_code = StatusCode::Status200;

          data.set_data(machine, key, DataType::F32Value(value));
        }
        Err(_) => {
          response.status_code = StatusCode::Status400;
          response.set_content("Invalid value");
        }
      };

      request.respond(&response);
    },
  );

  server.start().unwrap();

  Ok(())
}
