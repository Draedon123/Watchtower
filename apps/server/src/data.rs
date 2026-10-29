use std::collections::{HashMap, VecDeque};
use std::fmt::Display;
use std::time::{SystemTime, UNIX_EPOCH};

pub enum DataType {
  F32Value(f32),
  Vec2U64Value((u64, u64)),
  Unknown(String),
}

impl DataType {
  pub fn get_type_string(&self) -> &str {
    match self {
      DataType::F32Value(_) => "f32",
      DataType::Vec2U64Value(_) => "vec2u64",
      DataType::Unknown(_) => "unknown",
    }
  }
}

impl Display for DataType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let type_string = self.get_type_string();

    match self {
      DataType::F32Value(value) => write!(f, "{type_string}\n{value}"),
      DataType::Vec2U64Value((value_1, value_2)) => write!(f, "{type_string}\n{value_1},{value_2}"),
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

pub struct DataEntry {
  pub data: DataType,
  pub timestamp: u64,
}

impl DataEntry {
  fn get_current_time() -> u64 {
    SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_secs()
  }
}

impl From<f32> for DataEntry {
  fn from(value: f32) -> DataEntry {
    DataEntry {
      data: DataType::F32Value(value),
      timestamp: DataEntry::get_current_time(),
    }
  }
}

impl From<(u64, u64)> for DataEntry {
  fn from(value: (u64, u64)) -> DataEntry {
    DataEntry {
      data: DataType::Vec2U64Value(value),
      timestamp: DataEntry::get_current_time(),
    }
  }
}

impl Display for DataEntry {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}\n{}", self.data, self.timestamp)
  }
}

pub struct Data {
  machine_data: HashMap<String, HashMap<String, VecDeque<DataEntry>>>,
  machine_by_connection: HashMap<String, String>,
  max_values_per_key: usize,
}

type DataResult<T> = std::result::Result<T, DataResultError>;

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

impl Data {
  pub fn new(max_values_per_key: usize) -> Data {
    Data {
      machine_data: HashMap::new(),
      machine_by_connection: HashMap::new(),
      max_values_per_key,
    }
  }

  pub fn set_data(
    &mut self,
    machine: &str,
    data_key: &str,
    data_value: DataType,
  ) -> Result<(), String> {
    let values = self
      .machine_data
      .entry(machine.to_string())
      .or_insert(HashMap::new())
      .entry(data_key.to_string())
      .or_insert(VecDeque::with_capacity(self.max_values_per_key));

    if values.len() >= self.max_values_per_key {
      values.pop_back();
    }

    let data_entry = DataEntry {
      data: data_value,
      timestamp: DataEntry::get_current_time(),
    };

    values.push_front(data_entry);
    Ok(())
  }

  pub fn get_machine_data_recent(&self, machine: &str, data_key: &str) -> DataResult<&DataEntry> {
    Ok(
      self
        .get_machine_data_all(machine, data_key)?
        .front()
        .ok_or(DataResultError::NoValuesError(data_key.to_string()))?,
    )
  }

  pub fn get_machine_data_all(
    &self,
    machine: &str,
    data_key: &str,
  ) -> DataResult<&VecDeque<DataEntry>> {
    Ok(
      self
        .machine_data
        .get(machine)
        .ok_or(DataResultError::MachineNotFound(machine.to_string()))?
        .get(data_key)
        .ok_or(DataResultError::KeyNotFound(data_key.to_string()))?,
    )
  }

  pub fn list_machines(&self) -> Vec<&String> {
    self.machine_data.keys().collect()
  }

  pub fn register_machine(&mut self, ip: String, machine: &str) -> bool {
    let machine = machine.to_string();

    self.machine_by_connection.insert(ip, machine.clone());

    if !self.machine_data.contains_key(&machine) {
      self.machine_data.insert(machine, HashMap::new());
      true
    } else {
      false
    }
  }

  pub fn unregister_machine(&mut self, ip: &str) -> Option<String> {
    let machine = self.machine_by_connection.remove(ip);

    match machine {
      Some(machine) => {
        self.machine_data.remove(&machine);
        Some(machine)
      }
      None => None,
    }
  }
}
