use std::{env, fs::read_to_string, thread::sleep, time};

use http::request::Request;

use crate::config::Config;

mod config;

fn main() {
  let args: Vec<String> = env::args().collect();
  let config = Config::new(&args);

  let hostname = get_hostname();
  let address = format!("{}", config.watchtower_url);
  let mut connection = Request::new(&address);

  connection.path = String::from(format!("/set/{}", hostname));
  connection.method = http::method::HTTPMethod::Post();
  connection.set_content(&config.password);
  connection.send();

  let (mut last_total, mut last_idle) = read_cpu_times();

  loop {
    sleep(time::Duration::from_millis(1000));

    let (new_total, new_idle) = read_cpu_times();

    let delta_total = new_total - last_total;
    let delta_idle = new_idle - last_idle;
    let delta_active = delta_total - delta_idle;

    let usage = (delta_active as f32) / (delta_total as f32);

    last_total = new_total;
    last_idle = new_idle;

    connection = Request::new(&address);
    connection.method = http::method::HTTPMethod::Post();
    connection.path = String::from(format!("/set/{}/cpu_usage", hostname));
    connection.set_content(&format!("{}\n{}", config.password, usage));
    connection.send();
  }
}

fn get_hostname() -> String {
  read_to_string("/etc/hostname").unwrap().trim().to_string()
}

fn read_cpu_times() -> (u64, u64) {
  let stats = read_to_string("/proc/stat").unwrap();
  let cpu_line = stats.lines().find(|line| line.starts_with("cpu ")).unwrap();
  let values: Vec<_> = cpu_line
    .split_whitespace()
    .skip(1)
    .map(|value| value.parse::<u64>().unwrap())
    .collect();

  let total_time = values.iter().take(8).sum();
  // idle and iowait
  let idle_time = values[3] + values[4];

  (total_time, idle_time)
}
