use config::Config;
use server::Server;
use std::env;
use std::sync::{Arc, RwLock};

mod config;
mod data;
mod server;

use http::method::HTTPMethod;
use http::status_codes::StatusCode;

use crate::data::Data;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let args: Vec<String> = env::args().collect();
  let config = Config::new(&args);

  let mut server = Server::new(&config).unwrap();
  let data = Arc::new(RwLock::new(Data::new(10)));

  data
    .write()
    .unwrap()
    .set_data("test-machine", "cpu-temp", data::DataType::F32Value(45.0));

  server.route("/", HTTPMethod::Get(), |request, response| {
    response.status_code = StatusCode::Status200;
    request.respond(response);
  });

  let cpu_data = data.clone();
  server.route("/cputemp", HTTPMethod::Post(), move |request, response| {
    response
      .headers
      .set_header("Access-Control-Allow-Origin", "*");

    match &request.content {
      Some(content) => {
        let machine = content.as_str();
        let temperature_data = cpu_data.read().unwrap();
        let temperature = temperature_data.get_machine_data_recent(machine, "cpu-temp");

        match temperature {
          Some(data_entry) => {
            response.set_content(&data_entry.data.to_string());
            response.status_code = StatusCode::Status200;
          }
          None => {
            response.set_content("Machine not found");
            response.status_code = StatusCode::Status400;
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
  });

  server.start().unwrap();

  Ok(())
}
