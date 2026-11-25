use crate::{bit_vec::BitVec, split_slice::SplitSlice};

const ENCODING_ALPHABET: [char; 64] = [
  'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O',
  'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd',
  'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
  't', 'u', 'v', 'w', 'x', 'y', 'z', '0', '1', '2', '3', '4', '5', '6', '7',
  '8', '9', '+', '/',
];

// First 123 values in th UTF8 table and their corresponding base64 decoded sextet
const DECODING_TABLE: [u8; 123] = [
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 62, 0, 0, 0, 63, 52, 53,
  54, 55, 56, 57, 58, 59, 60, 61, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7,
  8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 0, 0,
  0, 0, 0, 0, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41,
  42, 43, 44, 45, 46, 47, 48, 49, 50, 51,
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
    let sextet = bits.get_byte_unaligned(i * 6).unwrap() >> 2;
    let encoded_character = ENCODING_ALPHABET[sextet as usize];

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

pub fn decode(encoded: &str) -> Vec<u8> {
  if encoded.is_empty() {
    return Vec::new();
  };

  let input = BitVec::from_bytes(encoded.as_bytes());
  let mut output_length = encoded.len() / 4 * 3;
  let mut input_length_unpadded = encoded.len();

  if input.get_byte_aligned(encoded.len() - 1).unwrap() == b'=' {
    output_length -= 1;
    input_length_unpadded -= 1;
  }

  if input.get_byte_aligned(encoded.len() - 2).unwrap() == b'=' {
    output_length -= 1;
    input_length_unpadded -= 1;
  }

  let mut output = BitVec::with_length(output_length * u8::BITS as usize);

  for i in 0..input_length_unpadded {
    let byte = input.get_byte_aligned(i).unwrap();
    let sextet = DECODING_TABLE[byte as usize];

    output.set_bits(i * 6, sextet << 2, 6);
  }

  let mut output = output.into_vec();

  output.truncate(output_length);

  output
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

  const EXPECT_VALID_UTF8: &str = "Decoding encoded utf8 should yield utf8";
  #[test]
  fn decode_empty() {
    assert_eq!(String::from_utf8(decode("")).expect(EXPECT_VALID_UTF8), "");
  }

  #[test]
  fn decode_2_padding() {
    assert_eq!(
      String::from_utf8(decode("Zg==")).expect(EXPECT_VALID_UTF8),
      "f"
    );
    assert_eq!(
      String::from_utf8(decode("Zm9vYg==")).expect(EXPECT_VALID_UTF8),
      "foob"
    );
  }

  #[test]
  fn decode_1_padding() {
    assert_eq!(
      String::from_utf8(decode("Zm8=")).expect(EXPECT_VALID_UTF8),
      "fo"
    );
    assert_eq!(
      String::from_utf8(decode("Zm9vYmE=")).expect(EXPECT_VALID_UTF8),
      "fooba"
    );
  }

  #[test]
  fn decode_no_padding() {
    assert_eq!(
      String::from_utf8(decode("Zm9v")).expect(EXPECT_VALID_UTF8),
      "foo"
    );
    assert_eq!(
      String::from_utf8(decode("Zm9vYmFy")).expect(EXPECT_VALID_UTF8),
      "foobar"
    );
  }
}
