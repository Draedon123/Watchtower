use std::fs::read_to_string;

use http::request::Request;

use crate::{config::Config, system::hostname::get_hostname};

pub struct CpuUsage {
  last_total: u64,
  last_idle: u64,
}

impl CpuUsage {
  pub fn new() -> CpuUsage {
    let (total, idle) = CpuUsage::read_cpu_times();

    CpuUsage {
      last_total: total,
      last_idle: idle,
    }
  }

  pub fn get_usage(&mut self) -> f32 {
    let (new_total, new_idle) = CpuUsage::read_cpu_times();

    let delta_total = new_total - self.last_total;
    let delta_idle = new_idle - self.last_idle;
    let delta_active = delta_total - delta_idle;

    let usage = (delta_active as f32) / (delta_total as f32);

    self.last_total = new_total;
    self.last_idle = new_idle;

    usage
  }

  pub fn send_to_watchtower(&mut self, config: &Config) {
    let mut connection = Request::new_auto_handle_errors(&config.watchtower_url);
    let hostname = get_hostname();

    if let None = connection.stream {
      eprintln!("Failed to send cpu usage to watchtower");
      return;
    };

    connection.method = http::method::HTTPMethod::Post();
    connection.path = String::from(format!("/set/{}/cpu_usage", hostname));
    connection
      .headers
      .set_header("Host", &config.watchtower_url);
    connection.set_content(&format!("{}\n{}", config.password, self.get_usage()));
    connection.send();
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
}
