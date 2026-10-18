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

  pub fn get_machine_data_recent(&self, machine: &str, data_key: &str) -> Option<&DataEntry> {
    self.machine_data.get(machine)?.get(data_key)?.front()
  }

  pub fn get_machine_data_all(
    &self,
    machine: &str,
    data_key: &str,
  ) -> Option<&VecDeque<DataEntry>> {
    self.machine_data.get(machine)?.get(data_key)
  }
}
