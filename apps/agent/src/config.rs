use config_parser::parse_args;

pub enum RunMode {
  Normal,
  Test(usize),
  Help,
  Unknown(String),
}

pub struct Config {
  pub password: String,
  pub watchtower_address: String,
  pub mode: RunMode,
}

impl Config {
  pub fn new(args: &[String]) -> Config {
    let (arguments, flags, _) = parse_args(args);

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
            println!("Failed to parse number of test instances \"{string}\": {error}");
            RunMode::Normal
          }
        },
        None => RunMode::Normal,
      }
    };

    Config {
      password,
      watchtower_address,
      mode,
    }
  }
}
