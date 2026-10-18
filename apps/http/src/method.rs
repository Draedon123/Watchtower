use std::fmt::Display;

#[derive(PartialEq)]
pub enum HTTPMethod {
  Get(),
  Post(),
  Put(),
  Delete(),
  Patch(),
  Unknown(String),
}

impl From<&str> for HTTPMethod {
  fn from(value: &str) -> Self {
    match value {
      "GET" => HTTPMethod::Get(),
      "POST" => HTTPMethod::Post(),
      "PUT" => HTTPMethod::Put(),
      "DELETE" => HTTPMethod::Delete(),
      "PATCH" => HTTPMethod::Patch(),
      &_ => HTTPMethod::Unknown(value.to_string()),
    }
  }
}

impl Display for HTTPMethod {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let formatted: &str = match self {
      HTTPMethod::Get() => "GET",
      HTTPMethod::Post() => "POST",
      HTTPMethod::Put() => "PUT",
      HTTPMethod::Delete() => "DELETE",
      HTTPMethod::Patch() => "PATCH",
      HTTPMethod::Unknown(method) => method,
    };

    f.write_str(formatted)?;

    Ok(())
  }
}
