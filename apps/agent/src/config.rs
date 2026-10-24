pub struct Config {
  pub password: String,
  pub watchtower_url: String,
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
          eprintln!("Argument \"{}\" is missing value", arg);
          break;
        }

        let value = &args[value_index];
        i += 1;

        match arg.as_str() {
          "--password" => {
            config.password = value.clone();
          }
          "--watchtower-url" => {
            config.watchtower_url = value.clone();
          }
          &_ => {
            eprintln!("Unknown argument \"{}\"", arg);
          }
        };
      }

      i += 1;
    }

    config
  }

  fn empty() -> Config {
    Config {
      password: String::new(),
      watchtower_url: String::new(),
    }
  }
}
