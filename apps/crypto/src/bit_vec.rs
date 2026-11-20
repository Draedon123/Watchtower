struct BitVec(Vec<u8>);

impl BitVec {
  pub fn new(capacity: usize) -> Self {
    BitVec(vec![0; f32::ceil(capacity as f32 / 8.0) as usize])
  }

  pub fn get(&self, bit: usize) -> Option<bool> {
    let byte = Self::get_containing_byte_index(bit);
    let bit_index = Self::get_bit_index(bit);

    self.0.get(byte).map(|byte| byte & (1 << bit_index) != 0)
  }

  pub fn set(&mut self, bit: usize, value: bool) {
    let byte = self.get_containing_byte_mut(bit);
    let bit_index = Self::get_bit_index(bit);

    if value {
      *byte |= 1 << bit_index
    } else {
      *byte &= !(1 << bit_index);
    }
  }

  fn get_containing_byte(&self, bit: usize) -> &u8 {
    let byte_index = Self::get_containing_byte_index(bit);
    &self.0[byte_index]
  }

  fn get_containing_byte_mut(&mut self, bit: usize) -> &mut u8 {
    let byte_index = Self::get_containing_byte_index(bit);
    &mut self.0[byte_index]
  }

  fn get_containing_byte_index(bit: usize) -> usize {
    bit / 8
  }

  fn get_bit_index(bit: usize) -> usize {
    7 - (bit % 8)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn get_and_set() {
    let mut bit_vec = BitVec::new(32);
    assert_eq!(bit_vec.0.len(), 4);

    assert_eq!(bit_vec.get(14), Some(false));
    assert_eq!(*bit_vec.get_containing_byte(14), 0u8);

    bit_vec.set(14, true);
    assert_eq!(bit_vec.get(14), Some(true));
    assert_eq!(*bit_vec.get_containing_byte(14), 2u8);

    bit_vec.set(14, false);
    assert_eq!(bit_vec.get(14), Some(false));
    assert_eq!(*bit_vec.get_containing_byte(14), 0u8);
  }
}
