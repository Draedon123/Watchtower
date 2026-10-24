use config_parser::parse_args;

pub struct Config {
  pub password: String,
  pub watchtower_url: String,
}

impl Config {
  pub fn new(args: &[String]) -> Config {
    let (arguments, _, _) = parse_args(args);

    let watchtower_url = match arguments.get("watchtower-url") {
      Some(string) => string.clone(),
      None => {
        eprintln!("No watchtower url specified. Set one with --watchtower-url URL");
        String::new()
      }
    };

    let password = match arguments.get("password") {
      Some(string) => string.clone(),
      None => String::new(),
    };

    Config {
      password,
      watchtower_url,
    }
  }
}
