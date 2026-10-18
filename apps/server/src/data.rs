use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

pub enum DataType {
  F32Value(f32),
}

impl ToString for DataType {
  fn to_string(&self) -> String {
    match self {
      DataType::F32Value(value) => value.to_string(),
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
      DataResultError::KeyNotFound(key) => f.write_fmt(format_args!("Key \"{}\" not found", key)),
      DataResultError::MachineNotFound(machine) => {
        f.write_fmt(format_args!("Machine \"{}\" not found", machine))
      }
      DataResultError::NoValuesError(key) => {
        f.write_fmt(format_args!("No values for key \"{}\" found", key))
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

  pub fn set_data(&mut self, machine: &str, data_key: &str, data_value: DataType) {
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
      timestamp: SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs(),
    };

    values.push_front(data_entry);
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
