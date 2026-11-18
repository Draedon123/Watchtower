use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::error::Error;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use http::http_status_codes::HttpStatusCode;

use crate::data::DataResultError::{
  KeyNotFound, MachineNotFound, NoValuesError,
};

#[derive(Clone)]
pub enum DataType {
  F32Value(f32),
  Vec2U64Value((u64, u64)),
  None,
  Unknown(String),
}

impl DataType {
  pub fn get_type_string(&self) -> &str {
    match self {
      Self::F32Value(_) => "f32",
      Self::Vec2U64Value(_) => "vec2u64",
      Self::None => "",
      Self::Unknown(_) => "unknown",
    }
  }

  pub fn serialise(&self) -> String {
    let type_string = self.get_type_string();

    match self {
      Self::F32Value(value) => format!("{type_string}\n{value}"),
      Self::Vec2U64Value((value_1, value_2)) => {
        format!("{type_string}\n{value_1},{value_2}")
      }
      Self::None => String::new(),
      Self::Unknown(value) => format!("{type_string}\n{value}"),
    }
  }
}

impl From<(&str, &str)> for DataType {
  fn from((value_type, value): (&str, &str)) -> Self {
    match value_type {
      "f32" => match value.parse::<f32>() {
        Ok(value) => DataType::F32Value(value),
        Err(_) => DataType::Unknown(value_type.to_owned()),
      },
      "vec2u64" => match value.split_once(",") {
        Some((value_1, value_2)) => {
          match (value_1.parse::<u64>(), value_2.parse::<u64>()) {
            (Ok(value_1), Ok(value_2)) => {
              DataType::Vec2U64Value((value_1, value_2))
            }
            _ => DataType::Unknown(value.to_owned()),
          }
        }
        None => DataType::Unknown(value.to_owned()),
      },
      _ => DataType::Unknown(value.to_owned()),
    }
  }
}

#[derive(Clone)]
pub struct DataEntry {
  pub data: DataType,
  pub timestamp: u64,
}

impl DataEntry {
  pub fn new(data: DataType) -> Self {
    Self {
      data,
      timestamp: DataEntry::get_current_time(),
    }
  }

  pub fn serialise(&self) -> String {
    format!("{}\n{}", self.data.serialise(), self.timestamp)
  }

  fn get_current_time() -> u64 {
    SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap_or_else(|error| {
        eprintln!("Failed to fetch system time: {error}. Returning UNIX_EPOCH");
        Duration::from_secs(0)
      })
      .as_secs()
  }
}

impl From<f32> for DataEntry {
  fn from(value: f32) -> Self {
    Self::new(DataType::F32Value(value))
  }
}

impl From<(u64, u64)> for DataEntry {
  fn from(value: (u64, u64)) -> Self {
    Self::new(DataType::Vec2U64Value(value))
  }
}

type DataResult<T> = std::result::Result<T, DataResultError>;

#[derive(Debug)]
pub enum DataResultError {
  MachineNotFound(String),
  KeyNotFound(String),
  NoValuesError(String),
}

impl Error for DataResultError {}

impl std::fmt::Display for DataResultError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      DataResultError::KeyNotFound(key) => write!(f, "Key \"{key}\" not found"),
      DataResultError::MachineNotFound(machine) => {
        write!(f, "Machine \"{machine}\" not found")
      }
      DataResultError::NoValuesError(key) => {
        write!(f, "No values for key \"{key}\" found")
      }
    }
  }
}

impl DataResultError {
  pub fn get_http_error_code(&self) -> HttpStatusCode {
    match *self {
      MachineNotFound(_) | KeyNotFound(_) => HttpStatusCode::Status404,
      // TODO: return entire vector and update status code
      NoValuesError(_) => HttpStatusCode::Status404,
    }
  }
}

pub struct Data {
  data_by_machine: HashMap<String, HashMap<String, DataEntry>>,
  data_by_key: HashMap<String, HashMap<String, DataEntry>>,
  machine_by_id: HashMap<u64, String>,
}

impl Data {
  pub fn new() -> Self {
    Self {
      data_by_machine: HashMap::new(),
      data_by_key: HashMap::new(),
      machine_by_id: HashMap::new(),
    }
  }

  pub fn set_data(
    &mut self,
    machine: &str,
    data_key: &str,
    data_value: DataType,
  ) -> Result<(), String> {
    let entry = self
      .data_by_machine
      .entry(machine.to_owned())
      .or_default()
      .entry(data_key.to_owned());

    let data_entry = DataEntry {
      data: data_value,
      timestamp: DataEntry::get_current_time(),
    };

    entry.insert_entry(data_entry.clone());

    self
      .data_by_key
      .entry(data_key.to_owned())
      .or_default()
      .entry(machine.to_owned())
      .insert_entry(data_entry);

    Ok(())
  }

  pub fn get_machine_data(
    &self,
    machine: &str,
    data_key: &str,
  ) -> DataResult<&DataEntry> {
    self
      .data_by_machine
      .get(machine)
      .ok_or(DataResultError::MachineNotFound(machine.to_owned()))?
      .get(data_key)
      .ok_or(DataResultError::KeyNotFound(data_key.to_owned()))
  }

  pub fn list_machines(&self) -> Vec<&String> {
    self.data_by_machine.keys().collect()
  }

  pub fn list_machine_keys(&self, machine: &str) -> DataResult<Vec<&str>> {
    Ok(
      self
        .data_by_machine
        .get(machine)
        .ok_or(DataResultError::MachineNotFound(machine.to_owned()))?
        .keys()
        .map(|string| &string[..])
        .collect(),
    )
  }

  pub fn register_machine(&mut self, request_id: u64, machine: &str) -> bool {
    let machine = machine.to_owned();

    self.machine_by_id.insert(request_id, machine.clone());

    if let Entry::Vacant(entry) = self.data_by_machine.entry(machine) {
      entry.insert(HashMap::new());
      true
    } else {
      false
    }
  }

  pub fn unregister_machine(&mut self, id: u64) -> Option<String> {
    let machine = self.machine_by_id.remove(&id);

    match machine {
      Some(machine) => {
        self.data_by_machine.remove(&machine);

        for machines in self.data_by_key.values_mut() {
          machines.remove(&machine);
        }

        Some(machine)
      }
      None => None,
    }
  }
}

impl Default for Data {
  fn default() -> Self {
    Self::new()
  }
}
