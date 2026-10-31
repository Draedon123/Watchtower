use std::{fmt::Display, fs::read_to_string};

use http::{client::Client, method::HTTPMethod};
use watchtower_server::data::DataEntry;

use crate::{config::Config, system::hostname::HostnameFetcher};

pub enum MemoryType {
  Physical,
  Swap,
}

impl MemoryType {
  fn total_memory_key(&self) -> &str {
    match self {
      MemoryType::Physical => "MemTotal",
      MemoryType::Swap => "SwapTotal",
    }
  }

  fn available_memory_key(&self) -> &str {
    match self {
      MemoryType::Physical => "MemAvailable",
      MemoryType::Swap => "SwapFree",
    }
  }

  fn watchtower_path(&self) -> &str {
    match self {
      MemoryType::Physical => "memory_physical",
      MemoryType::Swap => "memory_swap",
    }
  }
}

pub struct MemoryData {
  pub total_kib: u64,
  pub available_kib: u64,
}

impl Display for MemoryData {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}\n{}", self.available_kib, self.total_kib)
  }
}

pub struct Memory<'a> {
  pub memory_type: MemoryType,
  hostname_fetcher: &'a HostnameFetcher,
}

impl<'a> Memory<'a> {
  pub fn new(memory_type: MemoryType, hostname_fetcher: &'a HostnameFetcher) -> Memory<'a> {
    Memory {
      memory_type,
      hostname_fetcher,
    }
  }

  pub fn send_to_watchtower(&mut self, config: &Config, client: &mut Client) -> Result<(), String> {
    let data = match self.get_data()? {
      Some(data) => Some(data),
      None => None,
    };

    Memory::send_data_to_watchtower(
      &self.hostname_fetcher,
      &self.memory_type,
      data,
      config,
      client,
    )
  }

  pub fn send_data_to_watchtower(
    hostname_fetcher: &'a HostnameFetcher,
    memory_type: &MemoryType,
    data: Option<MemoryData>,
    config: &Config,
    client: &mut Client,
  ) -> Result<(), String> {
    let hostname = match hostname_fetcher.get_hostname() {
      Ok(hostname) => hostname,
      Err(error) => {
        return Err(format!("Failed to get device hostname: {error}"));
      }
    };

    let data = match data {
      Some(data) => (DataEntry::from((data.available_kib, data.total_kib))).to_string(),
      None => String::new(),
    };

    let mut connection = client.request(
      HTTPMethod::Post(),
      &format!(
        "{}/set/{hostname}/{}",
        config.watchtower_address,
        memory_type.watchtower_path()
      ),
      None,
    )?;

    connection.set_content(&data);
    connection.send()?;

    Ok(())
  }

  pub fn get_data(&self) -> Result<Option<MemoryData>, String> {
    let meminfo = if let Ok(meminfo) = read_to_string("/proc/meminfo") {
      meminfo
    } else {
      return Err(String::from("Could not read from /proc/meminfo"));
    };

    let total_memory_key = self.memory_type.total_memory_key();
    let available_memory_key = self.memory_type.available_memory_key();

    let mut meminfo = meminfo.lines();

    let total_kib = match meminfo.find(|line| line.starts_with(total_memory_key)) {
      Some(line) => Memory::parse_meminfo_line(line)?,
      None => return Ok(None),
    };

    let available_kib = match meminfo.find(|line| line.starts_with(available_memory_key)) {
      Some(line) => Memory::parse_meminfo_line(line)?,
      None => return Ok(None),
    };

    Ok(Some(MemoryData {
      total_kib,
      available_kib,
    }))
  }

  fn parse_meminfo_line(line: &str) -> Result<u64, String> {
    // [key, memory_kibibytes, "kB"]
    let parts: Vec<_> = line.split_whitespace().collect();

    if parts.len() != 3 {
      return Err(format!("Invalid /proc/meminfo line: {line}"));
    };

    let kibibytes = parts[1];
    match kibibytes.parse::<u64>() {
      Ok(value) => Ok(value),
      Err(error) => Err(format!(
        "Could not parse memory quantity \"{kibibytes}\" into u64: {error}"
      )),
    }
  }
}
