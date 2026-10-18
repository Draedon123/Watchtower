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

  let mut data_writer = data.write().unwrap();

  data_writer.set_data("test-machine", "cputemp", DataType::F32Value(45.0));
  data_writer.set_data("test-machine", "cpufreq", DataType::F32Value(1300000000.0));

  drop(data_writer);

  server.route("/", HTTPMethod::Get(), |request, response| {
    response.status_code = StatusCode::Status200;
    request.respond(response);
  });

  let data_get = data.clone();
  server.route(
    "/get/[key]",
    HTTPMethod::Post(),
    move |request, response| {
      response
        .headers
        .set_header("Access-Control-Allow-Origin", "*");

      match &request.http_request.content {
        Some(content) => {
          let machine = content.as_str();
          let all_data = data_get.read().unwrap();
          let recent_data =
            all_data.get_machine_data_recent(machine, request.slugs.get("key").unwrap());

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
        }
        None => {
          response.status_code = StatusCode::Status400;
          response.set_content("No machine specified");
          request.respond(response);
        }
      }
    },
  );

  server.start().unwrap();

  Ok(())
}
