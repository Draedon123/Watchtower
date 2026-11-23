pub struct SplitSlice<'a> {
  immutable: &'a [u8],
  mutable: Vec<u8>,
}

impl<'a> SplitSlice<'a> {
  pub fn new(immutable: &'a [u8], mutable: Vec<u8>) -> Self {
    Self { immutable, mutable }
  }

  pub fn with_capacity(immutable: &'a [u8], capacity: usize) -> Self {
    Self {
      immutable,
      mutable: Vec::with_capacity(capacity),
    }
  }

  pub fn with_length(immutable: &'a [u8], length: usize) -> Self {
    Self {
      immutable,
      mutable: vec![0; length],
    }
  }

  pub fn with_padding(immutable: &'a [u8], padding: &'a mut [u8]) -> Self {
    Self {
      immutable,
      mutable: Vec::from(padding),
    }
  }

  pub fn get(&self, index: usize) -> u8 {
    if index < self.immutable.len() {
      self.immutable[index]
    } else {
      self.mutable[index - self.immutable.len()]
    }
  }

  pub fn get_ref(&self, index: usize) -> &u8 {
    if index < self.immutable.len() {
      &self.immutable[index]
    } else {
      &self.mutable[index - self.immutable.len()]
    }
  }

  /// Panics if trying to obtain mutable reference of immutable data
  pub fn get_mut_ref(&mut self, index: usize) -> &mut u8 {
    if index < self.immutable.len() {
      panic!(
        "Tried to get mutable reference to immutable data (index {index} < self.immutable.len() {})",
        self.immutable.len()
      );
    } else {
      &mut self.mutable[index - self.immutable.len()]
    }
  }

  /// Panics if trying to modify immutable data
  pub fn set(&mut self, index: usize, value: u8) {
    if index < self.immutable.len() {
      panic!(
        "Tried to get mutate immutable data (index {index} < self.immutable.len() {})",
        self.immutable.len()
      );
    } else {
      self.mutable[index - self.immutable.len()] = value;
    }
  }

  pub fn push(&mut self, value: u8) {
    self.mutable.push(value);
  }

  pub fn resize(&mut self, length: usize, value: u8) {
    let padding_length = length - self.immutable.len();

    self.mutable.resize(padding_length, value);
  }

  pub fn len(&self) -> usize {
    self.immutable.len() + self.mutable.len()
  }

  pub fn is_empty(&self) -> bool {
    self.immutable.is_empty() && self.mutable.is_empty()
  }
}
