// https://datatracker.ietf.org/doc/html/rfc6455#section-7.4.1
#[derive(Debug, Clone)]
pub enum WsStatusCode {
  /// 1000 indicates a normal closure, meaning that the purpose for which the
  /// connection was established has been fulfilled
  Normal,
  /// 1001 indicates that an endpoint is "going away", such as a server going
  /// down or a browser having navigated away from a page
  GoingAway,
  /// 1002 indicates that an endpoint is terminating the connection due to a
  /// protocol error
  ProtocolError,
  /// 1003 indicates that an endpoint is terminating the connection because it
  /// has received a type of data it cannot accept (e.g., an endpoint that
  /// understands only text data MAY send this if it receives a binary message)
  UnacceptableData,
  /// 1005 is a reserved value and MUST NOT be set as a status code in a Close
  /// control frame by an endpoint. It is designated for use in applications
  /// expecting a status code to indicate that no status code was actually
  /// present
  NoStatus,
  /// 1006 is a reserved value and MUST NOT be set as a status code in a Close
  /// control frame by an endpoint. It is designated for use in applications
  /// expecting a status code to indicate that the connection was closed
  /// abnormally, e.g., without sending or receiving a Close control frame
  Abnormal,
  /// 1007 indicates that an endpoint is terminating the connection because it
  /// has received data within a message that was not consistent with the type
  /// of the message (e.g., non-UTF-8 [RFC3629] data within a text message)
  InconsistentData,
  /// 1008 indicates that an endpoint is terminating the connection because it
  /// has received a message that violates its policy. This is a generic status
  /// code that can be returned when there is no other more suitable status
  /// code (e.g., 1003 or 1009) or if there is a need to hide specific details
  /// about the policy
  PolicyViolation,
  /// 1009 indicates that an endpoint is terminating the connection because it
  /// has received a message that is too big for it to process
  MessageTooLarge,
  /// 1010 indicates that an endpoint (client) is terminating the connection
  /// because it has expected the server to negotiate one or more extension,
  /// but the server didn't return them in the response message of the
  /// WebSocket handshake. The list of extensions that are needed SHOULD appear
  /// in the /reason/ part of the Close frame. Note that this status code is
  /// not used by the server, because it can fail the WebSocket handshake
  /// instead
  MissingExtension,
  /// 011 indicates that a server is terminating the connection because it
  /// encountered an unexpected condition that prevented it from fulfilling the
  /// request
  UnexpectedCondition,
  /// 1015 is a reserved value and MUST NOT be set as a status code in a Close
  /// control frame by an endpoint. It is designated for use in applications
  /// expecting a status code to indicate that the connection was closed due to
  /// a failure to perform a TLS handshake (e.g., the server certificate can't
  /// be verified)
  TlsFailure,
  /// Status codes in the range 0-999 are not used
  ///
  /// Status codes in the range 1000-2999 are reserved for definition by this
  /// protocol, its future revisions, and extensions specified in a permanent
  /// and readily available public specification
  ///
  /// Status codes in the range 3000-3999 are reserved for use by libraries,
  /// frameworks, and applications. These status codes are registered directly
  /// with IANA. The interpretation of these codes is undefined by this
  /// protocol
  ///
  /// Status codes in the range 4000-4999 are reserved for private use and thus
  /// can't be registered. Such codes can be used by prior agreements between
  /// WebSocket applications. The interpretation of these codes is undefined by
  /// this protocol
  Other(u16),
}

impl WsStatusCode {
  pub fn from_u16(code: u16) -> Self {
    match code {
      1000 => Self::Normal,
      1001 => Self::GoingAway,
      1002 => Self::ProtocolError,
      1003 => Self::UnacceptableData,
      1005 => Self::NoStatus,
      1006 => Self::Abnormal,
      1007 => Self::InconsistentData,
      1008 => Self::PolicyViolation,
      1009 => Self::MessageTooLarge,
      1010 => Self::MissingExtension,
      1011 => Self::UnexpectedCondition,
      1015 => Self::TlsFailure,
      code => Self::Other(code),
    }
  }

  pub fn to_u16(&self) -> u16 {
    match self {
      Self::Normal => 1000,
      Self::GoingAway => 1001,
      Self::ProtocolError => 1002,
      Self::UnacceptableData => 1003,
      Self::NoStatus => 1005,
      Self::Abnormal => 1006,
      Self::InconsistentData => 1007,
      Self::PolicyViolation => 1008,
      Self::MessageTooLarge => 1009,
      Self::MissingExtension => 1010,
      Self::UnexpectedCondition => 1011,
      Self::TlsFailure => 1015,
      Self::Other(code) => *code,
    }
  }
}
