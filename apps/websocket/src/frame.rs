use crypto::bit_vec::BitVec;

pub struct Frame {
  finished: bool,
  rsv1: bool,
  rsv2: bool,
  rsv3: bool,
  op_code: OpCode,
  mask: Option<u32>,
  payload: Vec<u8>,
}

// TODO: fragmented messages
impl Frame {
  pub fn into_buffer(mut self) -> Vec<u8> {
    let payload_length = PayloadLength::from_payload(&self.payload);
    let frame_size_bytes_no_payload: usize = 2
      + if self.mask.is_some() { 4 } else { 0 }
      + payload_length.serialised_length_bytes();

    let mut buffer =
      BitVec::with_length(frame_size_bytes_no_payload * u8::BITS as usize);

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
        byte_offset += 2;
      }
      PayloadLength::Large(length) => {
        buffer.set_u64_be_aligned(byte_offset, length);
        byte_offset += 8;
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

  pub fn from_string(value: String, mask: Option<u32>) -> Self {
    Frame {
      finished: true,
      rsv1: false,
      rsv2: false,
      rsv3: false,
      op_code: OpCode::Text,
      mask,
      payload: value.into_bytes(),
    }
  }

  pub fn from_bytes(bytes: Vec<u8>, mask: Option<u32>) -> Self {
    Frame {
      finished: true,
      rsv1: false,
      rsv2: false,
      rsv3: false,
      op_code: OpCode::Binary,
      mask,
      payload: bytes,
    }
  }

  // TODO: non-string errors
  pub fn close(
    payload: Option<(u16, Option<String>)>,
    mask: Option<u32>,
  ) -> Result<Self, String> {
    if let Some((_, Some(message))) = &payload
      && message.len() > 123
    {
      return Err(String::from("Message longer than 123 bytes"));
    };

    let payload = payload.map_or_default(|(status_code, reason)| {
      let reason = reason.unwrap_or_default();
      let status_code_bytes = status_code.to_be_bytes();

      let mut payload = Vec::<u8>::with_capacity(2 + reason.len());

      payload.push(status_code_bytes[0]);
      payload.push(status_code_bytes[1]);
      payload.extend_from_slice(reason.as_bytes());

      payload
    });

    Ok(Frame {
      finished: true,
      rsv1: false,
      rsv2: false,
      rsv3: false,
      op_code: OpCode::Ping,
      mask,
      payload,
    })
  }

  pub fn ping(
    payload: Option<Vec<u8>>,
    mask: Option<u32>,
  ) -> Result<Self, String> {
    if let Some(payload) = &payload
      && payload.len() > 125
    {
      return Err(String::from("Payload longer than 123 bytes"));
    };

    Ok(Frame {
      finished: true,
      rsv1: false,
      rsv2: false,
      rsv3: false,
      op_code: OpCode::Ping,
      mask,
      payload: payload.unwrap_or_default(),
    })
  }

  pub fn pong(
    payload: Option<Vec<u8>>,
    mask: Option<u32>,
  ) -> Result<Self, String> {
    if let Some(payload) = &payload
      && payload.len() > 125
    {
      return Err(String::from("Payload longer than 123 bytes"));
    };

    Ok(Frame {
      finished: true,
      rsv1: false,
      rsv2: false,
      rsv3: false,
      op_code: OpCode::Pong,
      mask,
      payload: payload.unwrap_or_default(),
    })
  }

  pub fn declare_fragment_start(&mut self) {
    if self.op_code.is_control() {
      panic!("Cannot have fragmented control frame");
    };

    self.finished = false;
  }

  pub fn declare_fragment_middle(&mut self) {
    if self.op_code.is_control() {
      panic!("Cannot have fragmented control frame");
    };

    self.finished = false;
    self.op_code = OpCode::Continuation;
  }

  pub fn declare_fragment_end(&mut self) {
    if self.op_code.is_control() {
      panic!("Cannot have fragmented control frame");
    };

    self.finished = true;
    self.op_code = OpCode::Continuation;
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

// TODO: combine with payload type
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

  pub fn is_control(&self) -> bool {
    matches!(
      self,
      Self::Close | Self::Ping | Self::Pong | Self::Control(_)
    )
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

  pub fn serialised_length_bytes(&self) -> usize {
    match self {
      Self::Small(_) => 0,
      Self::Medium(_) => 2,
      Self::Large(_) => 8,
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

#[cfg(test)]
mod tests {
  use super::*;

  // https://datatracker.ietf.org/doc/html/rfc6455#section-5.7

  #[test]
  fn single_frame_unmasked_text() {
    let message = String::from("Hello");
    let frame = Frame::from_string(message, None);

    assert_eq!(
      frame.into_buffer(),
      vec![0x81, 0x05, 0x48, 0x65, 0x6c, 0x6c, 0x6f]
    );
  }

  #[test]
  fn single_frame_masked_text() {
    let mask = 0x37fa213du32;
    let message = String::from("Hello");

    let frame = Frame::from_string(message, Some(mask));

    assert_eq!(
      frame.into_buffer(),
      vec![
        0x81, 0x85, 0x37, 0xfa, 0x21, 0x3d, 0x7f, 0x9f, 0x4d, 0x51, 0x58
      ]
    );
  }

  #[test]
  fn fragmented_unmasked_text() {
    let mut fragment_1 = Frame::from_string(String::from("Hel"), None);
    let mut fragment_2 = Frame::from_string(String::from("lo"), None);

    fragment_1.declare_fragment_start();
    fragment_2.declare_fragment_end();

    assert_eq!(fragment_1.into_buffer(), vec![0x01, 0x03, 0x48, 0x65, 0x6c]);
    assert_eq!(fragment_2.into_buffer(), vec![0x80, 0x02, 0x6c, 0x6f]);
  }

  #[test]
  fn unmasked_ping() {
    let message = String::from("Hello");
    let frame = Frame::ping(Some(message.into_bytes()), None)
      .expect("Message is less than 126 bytes");

    assert_eq!(
      frame.into_buffer(),
      vec![0x89, 0x05, 0x48, 0x65, 0x6c, 0x6c, 0x6f]
    );
  }

  #[test]
  fn masked_pong() {
    let mask = 0x37fa213du32;
    let message = String::from("Hello");
    let frame = Frame::pong(Some(message.into_bytes()), Some(mask))
      .expect("Message is less than 126 bytes");

    assert_eq!(
      frame.into_buffer(),
      vec![
        0x8a, 0x85, 0x37, 0xfa, 0x21, 0x3d, 0x7f, 0x9f, 0x4d, 0x51, 0x58
      ]
    );
  }

  #[test]
  fn single_frame_256_byte_binary_unmasked() {
    let message = vec![0x67u8; 256];
    let mut expected_frame = vec![0x82, 0x7E, 0x01, 0x00];

    expected_frame.extend_from_slice(&message);

    let frame = Frame::from_bytes(message, None);
    let frame = frame.into_buffer();

    assert_eq!(frame.len(), 4 + 256);
    assert_eq!(frame, expected_frame);
  }

  #[test]
  fn single_frame_64_kib_binary_unmasked() {
    let message = vec![0x67u8; 65536];
    let mut expected_frame =
      vec![0x82, 0x7F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00];

    expected_frame.extend_from_slice(&message);

    let frame = Frame::from_bytes(message, None);
    let frame = frame.into_buffer();

    assert_eq!(frame.len(), 10 + 65536);
    assert_eq!(frame[..10], expected_frame[..10]);
    assert_eq!(
      frame[frame.len() - 10..],
      expected_frame[expected_frame.len() - 10..]
    );
  }
}
