const DEFAULT_PORT: u16 = 9824;

pub struct Config {
  pub port: u16,
  pub password: String,
}

impl Config {
  pub fn new(args: &[String]) -> Config {
    let mut config = Config::empty();

    let mut i: usize = 0;
    while i < args.len() {
      let arg = &args[i];

      if arg.starts_with("--") {
        let value_index = i + 1;

        if value_index >= args.len() {
          println!("Argument \"{}\" is missing value", arg);
          break;
        }

        let value = &args[value_index];
        i += 1;

        match arg.as_str() {
          "--port" => {
            let port = value.parse::<u16>().unwrap_or_else(|_| {
              println!("Failed to parse port \"{}\"", value);
              DEFAULT_PORT
            });

            config.port = port;
          }
          "--password" => {
            config.password = value.clone();
          }
          &_ => {
            println!("Unknown argument \"{}\"", arg);
          }
        };
      }

      i += 1;
    }

    config
  }

  fn empty() -> Config {
    Config {
      port: DEFAULT_PORT,
      password: String::new(),
    }
  }
}
