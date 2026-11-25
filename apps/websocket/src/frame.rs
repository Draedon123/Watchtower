use crypto::{bit_vec::BitVec, split_slice::SplitSlice};

use crate::frame;

pub struct Frame {
  finished: bool,
  rsv1: bool,
  rsv2: bool,
  rsv3: bool,
  op_code: OpCode,
  mask: Option<u32>,
  payload: Vec<u8>,
}

impl Frame {
  pub fn new() -> Self {
    Frame {
      finished: true,
      rsv1: false,
      rsv2: false,
      rsv3: false,
      op_code: OpCode::Ping,
      mask: None,
      payload: Vec::new(),
    }
  }

  pub fn into_buffer(mut self) -> Vec<u8> {
    let payload_length = PayloadLength::from_payload(&self.payload);
    let frame_size_bytes: usize =
      2 + payload_length.serialised_length_bits() + self.payload.len();

    let mut buffer =
      BitVec::from_split_slice(SplitSlice::with_length(&[], frame_size_bytes));

    buffer.set(0, self.finished);
    buffer.set(1, self.rsv1);
    buffer.set(2, self.rsv2);
    buffer.set(3, self.rsv3);
    buffer.set_bits(4, self.op_code.as_byte() << 4, 4);
    buffer.set(8, self.mask.is_some());
    buffer.set_bits(9, payload_length.get_first_serialised_byte() << 1, 7);

    let mut byte_offset: usize = 2;

    match payload_length {
      PayloadLength::Small(_) => {}
      PayloadLength::Medium(length) => {
        buffer.set_u16_be_aligned(byte_offset, length);
        byte_offset += u16::BITS as usize / 8;
      }
      PayloadLength::Large(length) => {
        buffer.set_u64_be_aligned(byte_offset, length);
        byte_offset += u64::BITS as usize / 8;
      }
    }

    match self.mask {
      Some(mask) => {
        buffer.set_u32_be_aligned(byte_offset, mask);

        Self::mask_payload(&mut self.payload, mask);

        let mut buffer = buffer.into_vec();
        buffer.extend_from_slice(&self.payload);
        buffer
      }
      None => {
        let mut buffer = buffer.into_vec();
        buffer.extend_from_slice(&self.payload);
        buffer
      }
    }
  }

  fn mask_payload(payload: &mut [u8], mask: u32) {
    for (i, byte) in payload.iter_mut().enumerate() {
      let mask_octet = 3 - (i % 4);
      let mask_octet = mask >> (8 * mask_octet);

      *byte ^= mask_octet as u8;
    }
  }
}

impl Default for Frame {
  fn default() -> Self {
    Self::new()
  }
}

pub enum OpCode {
  Continuation,
  Text,
  Binary,
  Close,
  Ping,
  Pong,
  NonControl(u8),
  Control(u8),
}

impl OpCode {
  pub fn as_byte(&self) -> u8 {
    match self {
      Self::Continuation => 0x0,
      Self::Text => 0x1,
      Self::Binary => 0x2,
      Self::Close => 0x8,
      Self::Ping => 0x9,
      Self::Pong => 0xA,
      Self::NonControl(code) => *code,
      Self::Control(code) => *code,
    }
  }
}

enum PayloadLength {
  Small(u8),
  Medium(u16),
  Large(u64),
}

impl PayloadLength {
  pub fn from_payload(payload: &[u8]) -> Self {
    if payload.len() <= 125 {
      Self::Small(payload.len() as u8)
    } else if payload.len() >= 126 && payload.len() <= 0xffff {
      Self::Medium(payload.len() as u16)
    } else {
      Self::Large(payload.len() as u64)
    }
  }

  pub fn serialised_length_bits(&self) -> usize {
    match self {
      Self::Small(_) => 0,
      Self::Medium(_) => u16::BITS as usize,
      Self::Large(_) => u64::BITS as usize,
    }
  }

  pub fn get_first_serialised_byte(&self) -> u8 {
    match self {
      Self::Small(length) => *length,
      Self::Medium(_) => 126,
      Self::Large(_) => 127,
    }
  }
}
