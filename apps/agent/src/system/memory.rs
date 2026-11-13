use std::{fmt::Display, fs::read_to_string};

use watchtower_server::data::DataEntry;

use crate::watchtower_bridge::Bridge;

pub enum MemoryType {
  Physical,
  Swap,
}

impl MemoryType {
  pub fn total_memory_key(&self) -> &str {
    match self {
      MemoryType::Physical => "MemTotal",
      MemoryType::Swap => "SwapTotal",
    }
  }

  pub fn available_memory_key(&self) -> &str {
    match self {
      MemoryType::Physical => "MemAvailable",
      MemoryType::Swap => "SwapFree",
    }
  }

  pub fn watchtower_path(&self) -> &str {
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

pub struct Memory {
  pub memory_type: MemoryType,
}

impl MemoryType {
  pub fn get_memory_data(&self) -> Result<Option<MemoryData>, String> {
    let meminfo = if let Ok(meminfo) = read_to_string("/proc/meminfo") {
      meminfo
    } else {
      return Err(String::from("Could not read from /proc/meminfo"));
    };

    let total_memory_key = self.total_memory_key();
    let available_memory_key = self.available_memory_key();

    let mut meminfo = meminfo.lines();

    let total_kib = match meminfo.find(|line| line.starts_with(total_memory_key)) {
      Some(line) => parse_meminfo_line(line)?,
      None => return Ok(None),
    };

    let available_kib = match meminfo.find(|line| line.starts_with(available_memory_key)) {
      Some(line) => parse_meminfo_line(line)?,
      None => return Ok(None),
    };

    Ok(Some(MemoryData {
      total_kib,
      available_kib,
    }))
  }
}

impl Bridge for MemoryType {
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
