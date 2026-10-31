use config_parser::parse_args;

const DEFAULT_PORT: u16 = 9824;
const DEFAULT_THREAD_COUNT: usize = 4;

pub enum RunMode {
  Normal,
  Help,
  // Unknown(String),
}

pub struct Config {
  pub port: u16,
  pub password: String,
  pub num_threads: usize,
  pub mode: RunMode,
}

impl Config {
  pub fn new(args: &[String]) -> Config {
    let (arguments, flags, _) = parse_args(args);

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

    let num_threads = match arguments.get("threads") {
      Some(string) => string.parse::<usize>().unwrap_or_else(|_| {
        eprintln!("Failed to parse thread count \"{string}\"");
        DEFAULT_THREAD_COUNT
      }),
      None => DEFAULT_THREAD_COUNT,
    };

    let mode = if flags.contains("h") {
      RunMode::Help
    } else {
      RunMode::Normal
    };

    assert!(num_threads > 0, "Thread count must be at least 1");

    Config {
      port,
      password,
      num_threads,
      mode,
    }
  }
}
