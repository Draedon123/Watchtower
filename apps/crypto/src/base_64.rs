use crate::{bit_vec::BitVec, split_slice::SplitSlice};

const ENCODING_ALPHABET: [char; 64] = [
  'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O',
  'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd',
  'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
  't', 'u', 'v', 'w', 'x', 'y', 'z', '0', '1', '2', '3', '4', '5', '6', '7',
  '8', '9', '+', '/',
];

const PADDING: char = '=';

pub fn encode(raw: &[u8]) -> String {
  // every 6 bits of input data is turned into 8 bits (one character), or
  // equivalently, 3 bits -> 4 bits. there can also be 2 padding characters
  let output_len_no_padding = (4 * raw.len()).div_ceil(3);
  let mut encoded = String::with_capacity(output_len_no_padding + 2);
  let bits = SplitSlice::new(raw, vec![0]);
  let bits = BitVec::from_split_slice(bits);

  for i in 0..output_len_no_padding {
    // right-shift by 2 to only read the first 6 bits of the byte
    let byte = bits.get_byte_unaligned(i * 6) >> 2;
    let encoded_character = ENCODING_ALPHABET[byte as usize];

    encoded.push(encoded_character);
  }

  if raw.len() % 3 == 2 {
    encoded.push(PADDING);
  } else if raw.len() % 3 == 1 {
    encoded.push(PADDING);
    encoded.push(PADDING);
  };

  encoded
}

#[cfg(test)]
mod tests {
  use super::*;

  // https://www.rfc-editor.org/info/rfc4648/#section-10

  #[test]
  fn encode_empty() {
    assert_eq!(encode(b""), "");
  }

  #[test]
  fn encode_2_padding() {
    assert_eq!(encode(b"f"), "Zg==");
    assert_eq!(encode(b"foob"), "Zm9vYg==");
  }

  #[test]
  fn encode_1_padding() {
    assert_eq!(encode(b"fo"), "Zm8=");
    assert_eq!(encode(b"fooba"), "Zm9vYmE=");
  }

  #[test]
  fn encode_no_padding() {
    assert_eq!(encode(b"foo"), "Zm9v");
    assert_eq!(encode(b"foobar"), "Zm9vYmFy");
  }
}
