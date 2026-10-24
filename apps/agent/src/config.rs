use config_parser::parse_args;

pub struct Config {
  pub password: String,
  pub watchtower_address: String,
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

    Config {
      password,
      watchtower_address,
    }
  }
}
