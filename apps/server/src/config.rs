use config_parser::parse_args;

const DEFAULT_PORT: u16 = 9824;

pub struct Config {
  pub port: u16,
  pub password: String,
}

impl Config {
  pub fn new(args: &[String]) -> Config {
    let (arguments, _, _) = parse_args(args);

    let port = match arguments.get("port") {
      Some(string) => string.parse::<u16>().unwrap_or_else(|_| {
        eprintln!("Failed to parse port \"{string}\"");
        DEFAULT_PORT
      }),
      None => DEFAULT_PORT,
    };

    let password = match arguments.get("password") {
      Some(string) => string.clone(),
      None => String::new(),
    };

    Config { port, password }
  }
}
