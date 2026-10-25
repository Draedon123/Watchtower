use std::collections::{HashMap, VecDeque};
use std::fmt::Display;
use std::time::{SystemTime, UNIX_EPOCH};

pub enum DataType {
  F32Value(f32),
}

impl Display for DataType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      DataType::F32Value(value) => write!(f, "{value}"),
    }
  }
}

pub struct DataEntry {
  pub data: DataType,
  pub timestamp: u64,
}

pub struct Data {
  machine_data: HashMap<String, HashMap<String, VecDeque<DataEntry>>>,
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

    let timestamp = if let Ok(timestamp) = SystemTime::now().duration_since(UNIX_EPOCH) {
      timestamp
    } else {
      return Err(String::from("Failed to get current time"));
    };

    let data_entry = DataEntry {
      data: data_value,
      timestamp: timestamp.as_secs(),
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

  pub fn register_machine(&mut self, machine: &str) -> bool {
    let machine = machine.to_string();

    if !self.machine_data.contains_key(&machine) {
      self.machine_data.insert(machine, HashMap::new());
      true
    } else {
      false
    }
  }
}

impl Display for DataEntry {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}\n{}", self.data, self.timestamp)
  }
}
