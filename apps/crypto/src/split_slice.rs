#[derive(Debug)]
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

  pub fn get(&self, index: usize) -> Option<u8> {
    if index < self.immutable.len() {
      self.immutable.get(index).cloned()
    } else {
      self.mutable.get(index - self.immutable.len()).cloned()
    }
  }

  pub fn get_ref(&self, index: usize) -> Option<&u8> {
    if index < self.immutable.len() {
      self.immutable.get(index)
    } else {
      self.mutable.get(index - self.immutable.len())
    }
  }

  pub fn get_mut(&mut self, index: usize) -> Option<&mut u8> {
    if index < self.immutable.len() {
      None
    } else {
      self.mutable.get_mut(index - self.immutable.len())
    }
  }

  // TODO: differentiate between fail reasons
  pub fn set(&mut self, index: usize, value: u8) -> bool {
    if index < self.immutable.len() {
      false
    } else {
      let mutable_index = index - self.immutable.len();

      match self.mutable.get_mut(mutable_index) {
        Some(element) => {
          *element = value;
          true
        }
        None => false,
      }
    }
  }

  pub fn push(&mut self, value: u8) {
    self.mutable.push(value);
  }

  pub fn resize(&mut self, length: usize, value: u8) {
    if length < self.mutable.len() {
      panic!("Cannot shrink immutable data");
    };

    let padding_length = length - self.immutable.len();

    self.mutable.resize(padding_length, value);
  }

  pub fn len(&self) -> usize {
    self.immutable.len() + self.mutable.len()
  }

  pub fn is_empty(&self) -> bool {
    self.immutable.is_empty() && self.mutable.is_empty()
  }

  pub fn into_vec(self) -> Vec<u8> {
    let mut output =
      Vec::with_capacity(self.immutable.len() + self.mutable.len());

    output.extend_from_slice(self.immutable);
    output.extend_from_slice(&self.mutable);

    output
  }
}
