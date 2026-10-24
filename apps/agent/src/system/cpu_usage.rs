use std::fs::read_to_string;

use http::{client::Client, header::Headers, method::HTTPMethod};

use crate::{config::Config, system::hostname::get_hostname};

pub struct CpuUsage {
  last_total: u64,
  last_idle: u64,
}

impl CpuUsage {
  pub fn new() -> CpuUsage {
    let (total, idle) = CpuUsage::read_cpu_times().unwrap_or((0, 0));

    CpuUsage {
      last_total: total,
      last_idle: idle,
    }
  }

  pub fn get_usage(&mut self) -> Result<f32, String> {
    let (new_total, new_idle) = match CpuUsage::read_cpu_times() {
      Ok(cpu_times) => cpu_times,
      Err(error) => return Err(format!("Failed to get cpu usage: {error}")),
    };

    let delta_total = new_total - self.last_total;
    let delta_idle = new_idle - self.last_idle;
    let delta_active = delta_total - delta_idle;

    let usage = (delta_active as f32) / (delta_total as f32);

    self.last_total = new_total;
    self.last_idle = new_idle;

    Ok(usage)
  }

  pub fn send_to_watchtower(&mut self, config: &Config, client: &mut Client) -> Result<(), String> {
    let hostname = match get_hostname() {
      Ok(hostname) => hostname,
      Err(error) => {
        return Err(format!("Failed to get device hostname: {error}"));
      }
    };

    let mut headers = Headers::new();
    headers
      .set_header("Host", &config.watchtower_address)
      .set_header("Authorization", &config.password);

    let mut connection = client.request(
      HTTPMethod::Post(),
      &format!("{}/set/{hostname}/cpu_usage", config.watchtower_address),
      Some(headers),
    );

    let usage = self.get_usage()?;

    connection.set_content(&format!("{usage}"));
    connection.send()?;

    Ok(())
  }

  fn read_cpu_times() -> Result<(u64, u64), String> {
    let stats = match read_to_string("/proc/stat") {
      Ok(data) => data,
      Err(_) => return Err(String::from("Could not read from /proc/stat")),
    };
    let cpu_line = stats
      .lines()
      .find(|line| line.starts_with("cpu "))
      .ok_or(String::from("Could not find cpu line in /proc/stat"))?;
    let values: Vec<_> = cpu_line
      .split_whitespace()
      .skip(1)
      .map(|value| value.parse::<u64>().unwrap_or(0))
      .collect();

    let total_time = values.iter().take(8).sum();
    // idle and iowait
    let idle_time = values[3] + values[4];

    Ok((total_time, idle_time))
  }
}
