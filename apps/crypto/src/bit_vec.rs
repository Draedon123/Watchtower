use crate::split_slice::SplitSlice;

const U8_BITS: usize = 8;

macro_rules! impl_for_utype {
  // gbeu/gbea/sbeu/sbea = get/set big endian unaligned/aligned
  ($type: ident, $bytes: literal, $gbeu: ident, $gbea: ident, $sbeu: ident, $sbea: ident) => {
    impl<'a> BitVec<'a> {
      pub fn $gbeu(&self, bit_offset: usize) -> Option<$type> {
        if bit_offset + $bytes * U8_BITS >= self.len() {
          return None;
        };

        Some($type::from_be_bytes(std::array::from_fn::<_, $bytes, _>(
          |i| {
            self
              .get_byte_unaligned(bit_offset + i * U8_BITS)
              .expect("BitVec is long enough")
          },
        )))
      }

      pub fn $gbea(&self, byte_offset: usize) -> Option<$type> {
        if (byte_offset + $bytes) * U8_BITS >= self.len() {
          return None;
        };

        Some($type::from_be_bytes(std::array::from_fn::<_, $bytes, _>(
          |i| {
            self
              .get_byte_aligned(byte_offset + i)
              .expect("BitVec is long enough")
          },
        )))
      }

      pub fn $sbeu(&mut self, bit_offset: usize, value: $type) {
        let end = bit_offset + $type::BITS as usize;

        if end >= self.len() {
          self.resize(end);
        }

        for i in 0..$type::BITS as usize {
          self.set(
            bit_offset + i,
            value & (1 << ($type::BITS as usize - 1 - i)) != 0,
          );
        }
      }

      pub fn $sbea(&mut self, byte_offset: usize, value: $type) {
        let bytes = value.to_be_bytes();

        for i in 0..$bytes {
          self.bits.set(byte_offset + i, bytes[i]);
        }
      }
    }
  };
}

impl_for_utype!(
  u16,
  2,
  get_u16_be_unaligned,
  get_u16_be_aligned,
  set_u16_be_unaligned,
  set_u16_be_aligned
);

impl_for_utype!(
  u32,
  4,
  get_u32_be_unaligned,
  get_u32_be_aligned,
  set_u32_be_unaligned,
  set_u32_be_aligned
);

impl_for_utype!(
  u64,
  8,
  get_u64_be_unaligned,
  get_u64_be_aligned,
  set_u64_be_unaligned,
  set_u64_be_aligned
);

#[derive(Debug)]
pub struct BitVec<'a> {
  bits: SplitSlice<'a>,
  length: usize,
}

impl<'a> BitVec<'a> {
  pub fn from_bytes(bits: &'a [u8]) -> Self {
    let bit_length = bits.len() * U8_BITS;

    Self {
      bits: SplitSlice::new(bits, Vec::new()),
      length: bit_length,
    }
  }

  pub fn from_bytes_mut(bits: &'a mut [u8]) -> Self {
    let bit_length = bits.len() * U8_BITS;

    Self {
      bits: SplitSlice::new(&[], Vec::from(bits)),
      length: bit_length,
    }
  }

  pub fn from_split_slice(slice: SplitSlice<'a>) -> Self {
    let bit_length = slice.len() * U8_BITS;

    Self {
      bits: slice,
      length: bit_length,
    }
  }

  pub fn with_length(bit_length: usize) -> Self {
    Self {
      bits: SplitSlice::with_length(
        &[],
        bit_length.div_ceil(u8::BITS as usize),
      ),
      length: bit_length,
    }
  }

  pub fn get(&self, bit: usize) -> Option<bool> {
    let byte_index = Self::get_containing_byte_index(bit);
    let bit_index = Self::get_bit_index(bit);

    self
      .bits
      .get(byte_index)
      .map(|byte| byte & (1 << bit_index) != 0)
  }

  pub fn set(&mut self, bit: usize, value: bool) -> bool {
    let byte = self.get_containing_byte_mut(bit);

    match byte {
      Some(byte) => {
        let bit_index = Self::get_bit_index(bit);

        if value {
          *byte |= 1 << bit_index;
        } else {
          *byte &= !(1 << bit_index);
        }

        true
      }
      None => false,
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

  pub fn get_byte_unaligned(&self, bit_offset: usize) -> Option<u8> {
    let mut byte: u8 = 0;

    for i in 0..U8_BITS {
      let bit = self.get(bit_offset + i)?;

      byte |= (bit as u8) << Self::get_bit_index(i)
    }

    Some(byte)
  }

  pub fn get_byte_aligned(&self, index: usize) -> Option<u8> {
    self.bits.get(index)
  }

  pub fn set_byte_unaligned(&mut self, bit_offset: usize, value: u8) {
    self.set_bits(bit_offset, value, U8_BITS);
  }

  /// The first `num_bits` of `value` are written
  pub fn set_bits(&mut self, bit_offset: usize, value: u8, num_bits: usize) {
    let end = bit_offset + num_bits;

    if end >= self.len() {
      self.resize(end);
    }

    for i in 0..num_bits {
      self.set(bit_offset + i, value & (1 << Self::get_bit_index(i)) != 0);
    }
  }

  pub fn set_byte_aligned(&mut self, index: usize, value: u8) {
    self.bits.set(index, value);
  }

  pub fn get_containing_byte(&self, bit: usize) -> Option<u8> {
    let byte_index = Self::get_containing_byte_index(bit);
    self.bits.get(byte_index)
  }

  pub fn get_containing_byte_ref(&self, bit: usize) -> Option<&u8> {
    let byte_index = Self::get_containing_byte_index(bit);
    self.bits.get_ref(byte_index)
  }

  fn get_containing_byte_mut(&mut self, bit: usize) -> Option<&mut u8> {
    let byte_index = Self::get_containing_byte_index(bit);
    self.bits.get_mut(byte_index)
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

  pub fn into_vec(self) -> Vec<u8> {
    self.bits.into_vec()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn get_and_set_bit() {
    let mut data: Vec<u8> = vec![0; 16];
    let mut bit_vec = BitVec::from_bytes_mut(&mut data);

    assert_eq!(bit_vec.get(14), Some(false));
    assert_eq!(bit_vec.get_containing_byte(14), Some(0u8));

    bit_vec.set(14, true);
    assert_eq!(bit_vec.get(14), Some(true));
    assert_eq!(bit_vec.get_containing_byte(14), Some(2u8));

    bit_vec.set(14, false);
    assert_eq!(bit_vec.get(14), Some(false));
    assert_eq!(bit_vec.get_containing_byte(14), Some(0u8));
  }

  #[test]
  fn get_and_set_byte() {
    let mut data: Vec<u8> = vec![0; 16];
    let mut bit_vec = BitVec::from_bytes_mut(&mut data);

    let value: u8 = 0x4a;

    bit_vec.set_byte_unaligned(4, value);
    let read_value = bit_vec
      .get_byte_unaligned(4)
      .expect("BitVec should be long enough");

    assert_eq!(read_value, value, "got 0x{:x} == 0x{:x}", read_value, value);
  }

  #[test]
  fn get_and_set_u16() {
    let mut data: Vec<u8> = vec![0; 32];
    let mut bit_vec = BitVec::from_bytes_mut(&mut data);

    let value: u16 = 0x0123;

    bit_vec.set_u16_be_unaligned(4, value);
    let read_value = bit_vec
      .get_u16_be_unaligned(4)
      .expect("BitVec should be long enough");

    assert_eq!(read_value, value, "got 0x{:x} == 0x{:x}", read_value, value);
  }

  #[test]
  fn get_and_set_u32() {
    let mut data: Vec<u8> = vec![0; 64];
    let mut bit_vec = BitVec::from_bytes_mut(&mut data);

    let value: u32 = 0x01234567;

    bit_vec.set_u32_be_unaligned(4, value);
    let read_value = bit_vec
      .get_u32_be_unaligned(4)
      .expect("BitVec should be long enough");

    assert_eq!(read_value, value, "got 0x{:x} == 0x{:x}", read_value, value);
  }

  #[test]
  fn get_and_set_u64() {
    let mut data: Vec<u8> = vec![0; 128];
    let mut bit_vec = BitVec::from_bytes_mut(&mut data);

    let value: u64 = 0x0123456789abcdef;

    bit_vec.set_u64_be_unaligned(4, value);
    let read_value = bit_vec
      .get_u64_be_unaligned(4)
      .expect("BitVec should be long enough");

    assert_eq!(read_value, value, "got 0x{:x} == 0x{:x}", read_value, value);
  }
}
