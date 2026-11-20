use crate::bit_vec::BitVec;

struct Message {
  pub length: usize,
  pub bits: BitVec,
}

impl Message {
  pub fn from_bytes(bytes: Vec<u8>) -> Self {
    let length = bytes.len() * 8;
    let bits = BitVec::from_bytes(bytes);

    Self { length, bits }
  }
}

fn ch(x: u32, y: u32, z: u32) -> u32 {
  (x & y) ^ (!x & z)
}

fn parity(x: u32, y: u32, z: u32) -> u32 {
  x ^ y ^ z
}

fn maj(x: u32, y: u32, z: u32) -> u32 {
  (x & y) ^ (x & z) ^ (y & z)
}

fn f(x: u32, y: u32, z: u32, t: u8) -> u32 {
  if t <= 19 {
    ch(x, y, z)
  } else if t <= 39 {
    parity(x, y, z)
  } else if t <= 59 {
    maj(x, y, z)
  } else {
    parity(x, y, z)
  }
}

fn k(t: u32) -> u32 {
  if t <= 19 {
    0x5a827999
  } else if t <= 39 {
    0x6ed9eba1
  } else if t <= 59 {
    0x8f1bbcdc
  } else {
    0xca62c1d6
  }
}

fn pad(message: &mut Message) {
  let padding = 448 - (message.length % 512 + 1);

  message.bits.resize(message.length + padding + 1 + 64);
  message.bits.set(message.length, true);
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn pad_test() {
    // ascii
    let a: u8 = 0b01100001;
    let b: u8 = 0b01100010;
    let c: u8 = 0b01100011;

    let mut message = Message::from_bytes(vec![a, b, c]);
    pad(&mut message);

    assert_eq!(message.bits.len(), 512);
    assert_eq!(*message.bits.get_byte(0), a);
    assert_eq!(*message.bits.get_byte(1), b);
    assert_eq!(*message.bits.get_byte(2), c);
    assert_eq!(*message.bits.get_byte(3), 0b10000000);

    for i in 4..64usize {
      assert_eq!(*message.bits.get_byte(i), 0);
    }
  }
}
