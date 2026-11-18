use std::fs::read_to_string;

use watchtower_server::data::DataEntry;

use crate::watchtower_bridge::Bridge;

pub struct MemoryData {
  pub total_kib: u64,
  pub available_kib: u64,
}

impl MemoryData {
  pub fn serialise(&self) -> String {
    format!("{}\n{}", self.available_kib, self.total_kib)
  }
}

pub enum Memory {
  Physical,
  Swap,
}

impl Memory {
  pub fn total_memory_key(&self) -> String {
    match self {
      Self::Physical => String::from("MemTotal"),
      Self::Swap => String::from("SwapTotal"),
    }
  }

  pub fn available_memory_key(&self) -> String {
    match self {
      Self::Physical => String::from("MemAvailable"),
      Self::Swap => String::from("SwapFree"),
    }
  }

  pub fn watchtower_path(&self) -> String {
    match self {
      Self::Physical => String::from("memory_physical"),
      Self::Swap => String::from("memory_swap"),
    }
  }

  pub fn get_memory_data(&self) -> Result<Option<MemoryData>, String> {
    let meminfo = read_to_string("/proc/meminfo")
      .map_err(|error| format!("Could not read from /proc/meminfo: {error}"))?;
    let total_memory_key = self.total_memory_key();
    let available_memory_key = self.available_memory_key();

    let mut meminfo = meminfo.lines();

    let total_kib =
      match meminfo.find(|line| line.starts_with(&total_memory_key)) {
        Some(line) => Self::parse_meminfo_line(line)?,
        None => return Ok(None),
      };

    let available_kib =
      match meminfo.find(|line| line.starts_with(&available_memory_key)) {
        Some(line) => Self::parse_meminfo_line(line)?,
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
    kibibytes.parse::<u64>().map_err(|error| {
      format!(
        "Could not parse memory quantity \"{kibibytes}\" into u64: {error}"
      )
    })
  }
}

impl Bridge for Memory {
  fn get_data(&mut self) -> Result<(String, DataEntry), String> {
    Ok((
      String::from(self.watchtower_path()),
      match self.get_memory_data()? {
        Some(data) => DataEntry::from((data.available_kib, data.total_kib)),
        None => DataEntry::new(watchtower_server::data::DataType::None),
      },
    ))
  }
}
