use std::fs::read_to_string;

use http::{client::Client, method::HTTPMethod};
use watchtower_server::data::DataEntry;

use crate::{config::Config, system::hostname::HostnameFetcher};

pub struct CpuUsage<'a> {
  last_total: u64,
  last_idle: u64,
  hostname_fetcher: &'a HostnameFetcher,
}

impl<'a> CpuUsage<'a> {
  pub fn new(hostname_fetcher: &'a HostnameFetcher) -> CpuUsage<'a> {
    let (total, idle) = CpuUsage::read_cpu_times().unwrap_or((0, 0));

    CpuUsage {
      last_total: total,
      last_idle: idle,
      hostname_fetcher,
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
    let hostname = match self.hostname_fetcher.get_hostname() {
      Ok(hostname) => hostname,
      Err(error) => {
        return Err(format!("Failed to get device hostname: {error}"));
      }
    };

    let mut connection = client.request(
      HTTPMethod::Post(),
      &format!("{}/set/{hostname}/cpu_usage", config.watchtower_address),
      None,
    )?;

    let usage = self.get_usage()?;
    let data = DataEntry::from(usage);

    connection.set_content(&format!("{data}"));
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
