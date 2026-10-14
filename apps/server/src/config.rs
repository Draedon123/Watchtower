const DEFAULT_PORT: u16 = 9824;

pub struct Config {
  pub port: u16,
}

impl Config {
  pub fn new(args: &[String]) -> Config {
    let port_argument_index = args.iter().position(|arg| arg == "--port");
    let port: u16 = match port_argument_index {
      Some(index) => {
        let port_index = index + 1;

        if port_index >= args.len() {
          DEFAULT_PORT
        } else {
          let port_string = &args[port_index];
          let port = port_string.parse::<u16>().unwrap_or(DEFAULT_PORT);

          port
        }
      }
      None => DEFAULT_PORT,
    };

    Config { port }
  }
}
