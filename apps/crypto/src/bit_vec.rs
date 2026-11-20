pub struct BitVec {
  bits: Vec<u8>,
  length: usize,
}

impl BitVec {
  pub fn new(length: usize) -> Self {
    BitVec {
      bits: vec![0; Self::bytes_required(length)],
      length,
    }
  }

  pub fn from_bytes(bits: Vec<u8>) -> Self {
    let bit_length = bits.len() * 8;

    Self {
      bits,
      length: bit_length,
    }
  }

  pub fn get(&self, bit: usize) -> Option<bool> {
    let byte = Self::get_containing_byte_index(bit);
    let bit_index = Self::get_bit_index(bit);

    self.bits.get(byte).map(|byte| byte & (1 << bit_index) != 0)
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

  pub fn append(&mut self, bit: bool) {
    if self.len().is_multiple_of(8) {
      self.bits.push(0);
    }

    self.length += 1;

    self.set(self.length, bit);
  }

  pub fn len(&self) -> usize {
    self.length
  }

  pub fn is_empty(&self) -> bool {
    self.length == 0
  }

  pub fn resize(&mut self, length: usize) {
    self.length = length;
    self.bits.resize(Self::bytes_required(length), 0);
  }

  pub fn get_byte(&self, byte: usize) -> &u8 {
    &self.bits[byte]
  }

  pub fn write_byte(&mut self, index: usize, byte: u8) {
    self.bits[index] = byte;
  }

  pub fn get_u32_be(&self, index: usize) -> u32 {
    let byte_index = index * (32 / 8);

    u32::from_be_bytes([
      *self.get_byte(byte_index),
      *self.get_byte(byte_index + 1),
      *self.get_byte(byte_index + 2),
      *self.get_byte(byte_index + 3),
    ])
  }

  pub fn get_u64_be(&self, index: usize) -> u64 {
    let byte_index = index * (64 / 8);

    u64::from_be_bytes([
      *self.get_byte(byte_index),
      *self.get_byte(byte_index + 1),
      *self.get_byte(byte_index + 2),
      *self.get_byte(byte_index + 3),
      *self.get_byte(byte_index + 4),
      *self.get_byte(byte_index + 5),
      *self.get_byte(byte_index + 6),
      *self.get_byte(byte_index + 7),
    ])
  }

  pub fn write_u64_be(&mut self, value: u64, bit_offset: usize) {
    let end = bit_offset + 64;

    if end >= self.len() {
      self.resize(end);
    }

    for i in 0..64usize {
      self.set(bit_offset + i, value & (1 << (63 - i)) != 0);
    }
  }

  pub fn get_containing_byte(&self, bit: usize) -> &u8 {
    let byte_index = Self::get_containing_byte_index(bit);
    &self.bits[byte_index]
  }

  fn get_containing_byte_mut(&mut self, bit: usize) -> &mut u8 {
    let byte_index = Self::get_containing_byte_index(bit);
    &mut self.bits[byte_index]
  }

  pub fn get_containing_byte_index(bit: usize) -> usize {
    bit / 8
  }

  fn get_bit_index(bit: usize) -> usize {
    7 - (bit % 8)
  }

  fn bytes_required(bits: usize) -> usize {
    bits.div_ceil(8)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn get_and_set() {
    let mut bit_vec = BitVec::new(128);
    assert_eq!(bit_vec.bits.len(), 16);

    assert_eq!(bit_vec.get(14), Some(false));
    assert_eq!(*bit_vec.get_containing_byte(14), 0u8);

    bit_vec.set(14, true);
    assert_eq!(bit_vec.get(14), Some(true));
    assert_eq!(*bit_vec.get_containing_byte(14), 2u8);

    bit_vec.set(14, false);
    assert_eq!(bit_vec.get(14), Some(false));
    assert_eq!(*bit_vec.get_containing_byte(14), 0u8);

    bit_vec.write_u64_be(!0, 4);
    assert_eq!(bit_vec.bits[0], 0b00001111);
    assert_eq!(bit_vec.bits[1], 0b11111111);
    assert_eq!(bit_vec.get_u32_be(0), 0x0fffffff);
    assert_eq!(bit_vec.get_u64_be(0), 0x0fffffffffffffff);
  }
}
