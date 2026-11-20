const U8_BITS: usize = 8;
const U32_BITS: usize = 32;
const U64_BITS: usize = 64;

const U32_BYTES: usize = U32_BITS / 8;
const U64_BYTES: usize = U64_BITS / 8;

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
    let bit_length = bits.len() * U8_BITS;

    Self {
      bits,
      length: bit_length,
    }
  }

  pub fn get(&self, bit: usize) -> bool {
    let byte = Self::get_containing_byte_index(bit);
    let bit_index = Self::get_bit_index(bit);

    self.bits[byte] & (1 << bit_index) != 0
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
    if self.len().is_multiple_of(U8_BITS) {
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

  pub fn get_byte_unaligned(&self, bit_offset: usize) -> u8 {
    let mut byte: u8 = 0;

    for i in 0..U8_BITS {
      byte |= (self.get(bit_offset + i) as u8) << Self::get_bit_index(i);
    }

    byte
  }

  pub fn get_byte_aligned(&self, index: usize) -> &u8 {
    &self.bits[index]
  }

  pub fn set_byte_unaligned(&mut self, bit_offset: usize, value: u8) {
    let end = bit_offset + U8_BITS;

    if end >= self.len() {
      self.resize(end);
    }

    for i in 0..U8_BITS {
      self.set(bit_offset + i, value & (1 << Self::get_bit_index(i)) != 0);
    }
  }

  pub fn set_byte_aligned(&mut self, index: usize, value: u8) {
    self.bits[index] = value;
  }

  pub fn set_u32_be_aligned(&self, index: usize) -> u32 {
    let byte_index = index * U32_BYTES;

    u32::from_be_bytes([
      *self.get_byte_aligned(byte_index),
      *self.get_byte_aligned(byte_index + 1),
      *self.get_byte_aligned(byte_index + 2),
      *self.get_byte_aligned(byte_index + 3),
    ])
  }

  pub fn get_u64_be_unaligned(&self, bit_offset: usize) -> u64 {
    u64::from_be_bytes([
      self.get_byte_unaligned(bit_offset),
      self.get_byte_unaligned(bit_offset + U8_BITS),
      self.get_byte_unaligned(bit_offset + 2 * U8_BITS),
      self.get_byte_unaligned(bit_offset + 3 * U8_BITS),
      self.get_byte_unaligned(bit_offset + 4 * U8_BITS),
      self.get_byte_unaligned(bit_offset + 5 * U8_BITS),
      self.get_byte_unaligned(bit_offset + 6 * U8_BITS),
      self.get_byte_unaligned(bit_offset + 7 * U8_BITS),
    ])
  }

  pub fn get_u64_be_aligned(&self, index: usize) -> u64 {
    let byte_index = index * U64_BYTES;

    u64::from_be_bytes([
      *self.get_byte_aligned(byte_index),
      *self.get_byte_aligned(byte_index + 1),
      *self.get_byte_aligned(byte_index + 2),
      *self.get_byte_aligned(byte_index + 3),
      *self.get_byte_aligned(byte_index + 4),
      *self.get_byte_aligned(byte_index + 5),
      *self.get_byte_aligned(byte_index + 6),
      *self.get_byte_aligned(byte_index + 7),
    ])
  }

  pub fn set_u64_be_unaligned(&mut self, bit_offset: usize, value: u64) {
    let end = bit_offset + U64_BITS;

    if end >= self.len() {
      self.resize(end);
    }

    for i in 0..U64_BITS {
      self.set(bit_offset + i, value & (1 << (U64_BITS - 1 - i)) != 0);
    }
  }

  pub fn set_u64_be_aligned(&mut self, index: usize, value: u64) {
    let bytes = value.to_be_bytes();

    for i in 0..U64_BYTES {
      self.bits[index * U64_BYTES + i] = bytes[i]
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
    bit / U8_BITS
  }

  fn get_bit_index(bit: usize) -> usize {
    U8_BITS - 1 - (bit % U8_BITS)
  }

  fn bytes_required(bits: usize) -> usize {
    bits.div_ceil(U8_BITS)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn get_and_set_bit() {
    let mut bit_vec = BitVec::new(128);

    assert!(!bit_vec.get(14));
    assert_eq!(*bit_vec.get_containing_byte(14), 0u8);

    bit_vec.set(14, true);
    assert!(bit_vec.get(14));
    assert_eq!(*bit_vec.get_containing_byte(14), 2u8);

    bit_vec.set(14, false);
    assert!(!bit_vec.get(14));
    assert_eq!(*bit_vec.get_containing_byte(14), 0u8);
  }

  #[test]
  fn get_and_set_byte() {
    let mut bit_vec = BitVec::new(16);

    let value: u8 = 0x4a;

    bit_vec.set_byte_unaligned(4, value);
    let read_value = bit_vec.get_byte_unaligned(4);

    assert_eq!(read_value, value, "got 0x{:x} == 0x{:x}", read_value, value);
  }

  #[test]
  fn get_and_set_u64() {
    let mut bit_vec = BitVec::new(128);

    let value: u64 = 0x0123456789abcdef;

    bit_vec.set_u64_be_unaligned(4, value);
    let read_value = bit_vec.get_u64_be_unaligned(4);

    assert_eq!(read_value, value, "got 0x{:x} == 0x{:x}", read_value, value);
  }
}
