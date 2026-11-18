use config_parser::parse_args;

#[derive(Clone)]
pub enum RunMode {
  Normal,
  Test(usize),
  Help,
  Error,
}

#[derive(Clone)]
pub struct Config {
  pub password: String,
  pub watchtower_address: String,
  pub mode: RunMode,
}

impl Config {
  pub fn new(args: &[String]) -> Self {
    let (arguments, flags, _) = parse_args(args);

    let watchtower_address = arguments
      .get("watchtower-address")
      .map(|address| address.clone())
      .unwrap_or(String::from("127.0.0.1:9824"));

    let password = arguments
      .get("password")
      .map(|password| password.clone())
      .unwrap_or(String::new());

    let mode = if flags.contains("h") {
      RunMode::Help
    } else {
      match arguments.get("test-instances") {
        Some(string) => match string.parse::<usize>() {
          Ok(num_test_instances) => {
            if num_test_instances > 0 {
              RunMode::Test(num_test_instances)
            } else {
              RunMode::Normal
            }
          }
          Err(error) => {
            eprintln!(
              "Failed to parse number of test instances \"{string}\": {error}"
            );
            RunMode::Error
          }
        },
        None => RunMode::Normal,
      }
    };

    Self {
      password,
      watchtower_address,
      mode,
    }
  }
}
