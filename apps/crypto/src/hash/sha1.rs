use crate::bit_vec::BitVec;

const INITIAL_HASH: [u32; 5] =
  [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476, 0xc3d2e1f0];

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

fn f(x: u32, y: u32, z: u32, t: usize) -> u32 {
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

fn k(t: usize) -> u32 {
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
  let padding = (447 + 512 - (message.length % 512)) % 512;

  message.bits.resize(message.length + padding + 1 + 64);
  message.bits.set(message.length, true);
  message
    .bits
    .write_u64_be(message.length as u64, message.bits.len() - 64);
}

fn sha1(data: Vec<u8>) -> [u32; 5] {
  let mut message = Message::from_bytes(data);
  pad(&mut message);

  let mut h = INITIAL_HASH;

  for i in 0..message.bits.len() / 512 {
    let mut w = [0u32; 80];

    (0..80).for_each(|t| {
      w[t] = if t <= 15 {
        message.bits.get_u32_be(i * 16 + t)
      } else {
        (w[t - 3] ^ w[t - 8] ^ w[t - 14] ^ w[t - 16]).rotate_left(1)
      }
    });

    let mut a = h[0];
    let mut b = h[1];
    let mut c = h[2];
    let mut d = h[3];
    let mut e = h[4];

    (0..80usize).for_each(|t| {
      let temp = a
        .rotate_left(5)
        .wrapping_add(f(b, c, d, t))
        .wrapping_add(e)
        .wrapping_add(k(t))
        .wrapping_add(w[t]);

      e = d;
      d = c;
      c = b.rotate_left(30);
      b = a;
      a = temp;
    });

    h[0] = h[0].wrapping_add(a);
    h[1] = h[1].wrapping_add(b);
    h[2] = h[2].wrapping_add(c);
    h[3] = h[3].wrapping_add(d);
    h[4] = h[4].wrapping_add(e);
  }

  h
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn pad_test() {
    // ascii
    let a: u8 = 0x61;
    let b: u8 = 0x62;
    let c: u8 = 0x63;

    let mut message = Message::from_bytes(vec![a, b, c]);
    pad(&mut message);

    assert_eq!(message.bits.len(), 512);

    assert_eq!(message.bits.get_u64_be(0), 0x6162638000000000);
    assert_eq!(message.bits.get_u64_be(1), 0);
    assert_eq!(message.bits.get_u64_be(2), 0);
    assert_eq!(message.bits.get_u64_be(3), 0);
    assert_eq!(message.bits.get_u64_be(4), 0);
    assert_eq!(message.bits.get_u64_be(5), 0);
    assert_eq!(message.bits.get_u64_be(6), 0);
    assert_eq!(message.bits.get_u64_be(7), 8 * 3);
  }

  #[test]
  fn sha1_test() {
    // https://di-mgt.com.au/sha_testvectors.html
    assert_eq!(
      sha1(String::from("abc").into_bytes()),
      [0xa9993e36, 0x4706816a, 0xba3e2571, 0x7850c26c, 0x9cd0d89d]
    );

    assert_eq!(
      sha1(String::from("").into_bytes()),
      [0xda39a3ee, 0x5e6b4b0d, 0x3255bfef, 0x95601890, 0xafd80709]
    );

    assert_eq!(
      sha1(
        String::from(
          "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"
        )
        .into_bytes()
      ),
      [0x84983e44, 0x1c3bd26e, 0xbaae4aa1, 0xf95129e5, 0xe54670f1]
    );

    assert_eq!(sha1(
      String::from("abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu").into_bytes(),
    ),[0xa49b2446, 0xa02c645b, 0xf419f995, 0xb6709125, 0x3a04a259]);

    let mut one_million_a = Vec::<u8>::with_capacity(1000000);
    one_million_a.resize(1000000, b'a');

    assert_eq!(
      sha1(one_million_a),
      [0x34aa973c, 0xd4c4daa4, 0xf61eeb2b, 0xdbad2731, 0x6534016f]
    );
  }
}
