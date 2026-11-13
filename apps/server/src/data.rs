use std::collections::HashMap;
use std::fmt::Display;
use std::time::{SystemTime, UNIX_EPOCH};

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
      DataType::F32Value(_) => "f32",
      DataType::Vec2U64Value(_) => "vec2u64",
      DataType::None => "",
      DataType::Unknown(_) => "unknown",
    }
  }
}

impl Display for DataType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let type_string = self.get_type_string();

    match self {
      DataType::F32Value(value) => write!(f, "{type_string}\n{value}"),
      DataType::Vec2U64Value((value_1, value_2)) => {
        write!(f, "{type_string}\n{value_1},{value_2}")
      }
      DataType::None => Ok(()),
      DataType::Unknown(value) => write!(f, "{type_string}\n{value}"),
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
        Some((value_1, value_2)) => match (value_1.parse::<u64>(), value_2.parse::<u64>()) {
          (Ok(value_1), Ok(value_2)) => DataType::Vec2U64Value((value_1, value_2)),
          _ => DataType::Unknown(value.to_owned()),
        },
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
  pub fn new(data: DataType) -> DataEntry {
    DataEntry {
      data,
      timestamp: DataEntry::get_current_time(),
    }
  }

  fn get_current_time() -> u64 {
    SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_secs()
  }
}

impl From<f32> for DataEntry {
  fn from(value: f32) -> DataEntry {
    DataEntry::new(DataType::F32Value(value))
  }
}

impl From<(u64, u64)> for DataEntry {
  fn from(value: (u64, u64)) -> DataEntry {
    DataEntry::new(DataType::Vec2U64Value(value))
  }
}

impl Display for DataEntry {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}\n{}", self.data, self.timestamp)
  }
}

type DataResult<T> = std::result::Result<T, DataResultError>;

#[derive(Debug)]
pub enum DataResultError {
  MachineNotFound(String),
  KeyNotFound(String),
  NoValuesError(String),
}

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

pub struct Data {
  data_by_machine: HashMap<String, HashMap<String, DataEntry>>,
  data_by_key: HashMap<String, HashMap<String, DataEntry>>,
  machine_by_connection: HashMap<String, String>,
}

impl Data {
  pub fn new() -> Data {
    Data {
      data_by_machine: HashMap::new(),
      data_by_key: HashMap::new(),
      machine_by_connection: HashMap::new(),
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
      .entry(machine.to_string())
      .or_insert(HashMap::new())
      .entry(data_key.to_string());

    let data_entry = DataEntry {
      data: data_value,
      timestamp: DataEntry::get_current_time(),
    };

    entry.insert_entry(data_entry.clone());

    self
      .data_by_key
      .entry(data_key.to_string())
      .or_insert(HashMap::new())
      .entry(machine.to_string())
      .insert_entry(data_entry);

    Ok(())
  }

  pub fn get_machine_data(&self, machine: &str, data_key: &str) -> DataResult<&DataEntry> {
    Ok(
      self
        .data_by_machine
        .get(machine)
        .ok_or(DataResultError::MachineNotFound(machine.to_string()))?
        .get(data_key)
        .ok_or(DataResultError::KeyNotFound(data_key.to_string()))?,
    )
  }

  pub fn list_machines(&self) -> Vec<&String> {
    self.data_by_machine.keys().collect()
  }

  pub fn list_machine_keys(&self, machine: &str) -> DataResult<Vec<&str>> {
    Ok(
      self
        .data_by_machine
        .get(machine)
        .ok_or(DataResultError::MachineNotFound(machine.to_string()))?
        .keys()
        .map(|string| &string[..])
        .collect(),
    )
  }

  pub fn register_machine(&mut self, ip: String, machine: &str) -> bool {
    let machine = machine.to_string();

    self.machine_by_connection.insert(ip, machine.clone());

    if !self.data_by_machine.contains_key(&machine) {
      self.data_by_machine.insert(machine, HashMap::new());
      true
    } else {
      false
    }
  }

  pub fn unregister_machine(&mut self, ip: &str) -> Option<String> {
    let machine = self.machine_by_connection.remove(ip);

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
