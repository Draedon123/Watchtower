use std::fmt::Display;

#[derive(PartialEq, Clone, Debug)]
pub enum HttpMethod {
  Get(),
  Post(),
  Put(),
  Delete(),
  Patch(),
  Unknown(String),
}

impl From<&str> for HttpMethod {
  fn from(value: &str) -> Self {
    match value {
      "GET" => Self::Get(),
      "POST" => Self::Post(),
      "PUT" => Self::Put(),
      "DELETE" => Self::Delete(),
      "PATCH" => Self::Patch(),
      _ => Self::Unknown(value.to_owned()),
    }
  }
}

impl Display for HttpMethod {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let formatted: &str = match self {
      Self::Get() => "GET",
      Self::Post() => "POST",
      Self::Put() => "PUT",
      Self::Delete() => "DELETE",
      Self::Patch() => "PATCH",
      Self::Unknown(method) => method,
    };

    write!(f, "{}", formatted)
  }
}
