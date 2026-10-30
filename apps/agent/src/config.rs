use config_parser::parse_args;

pub enum RunMode {
  Normal,
  Test(usize),
  Unknown(String),
}

pub struct Config {
  pub password: String,
  pub watchtower_address: String,
  pub mode: RunMode,
}

impl Config {
  pub fn new(args: &[String]) -> Config {
    let (arguments, _, _) = parse_args(args);

    let watchtower_address = match arguments.get("watchtower-address") {
      Some(string) => string.clone(),
      None => {
        eprintln!("No watchtower address specified. Set one with --watchtower-address ADDRESS");
        String::new()
      }
    };

    let password = match arguments.get("password") {
      Some(string) => string.clone(),
      None => String::new(),
    };

    let mode = match arguments.get("test-instances") {
      Some(string) => match string.parse::<usize>() {
        Ok(num_test_instances) => RunMode::Test(num_test_instances),
        Err(_) => RunMode::Unknown(string.to_string()),
      },
      None => RunMode::Normal,
    };

    Config {
      password,
      watchtower_address,
      mode,
    }
  }
}
