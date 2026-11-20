use config_parser::parse_args;
use misc::uri::Uri;

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
  pub watchtower_uri: Uri,
  pub mode: RunMode,
}

impl Config {
  pub fn new(args: &[String]) -> Self {
    let (arguments, flags, _) = parse_args(args);

    let mut watchtower_address = arguments
      .get("watchtower-address")
      .cloned()
      .unwrap_or(String::from("http://127.0.0.1:9824"));

    if !watchtower_address.starts_with("http://") {
      println!("Automatically appending \"http://\" to watchtower address");
      watchtower_address = format!("http://{watchtower_address}");
    }

    let password = arguments.get("password").cloned().unwrap_or(String::new());

    let mut mode = if flags.contains("h") {
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

    let watchtower_uri = Uri::new(&watchtower_address).unwrap_or_else(|_| {
      eprintln!("Invalid watchtower address");
      mode = RunMode::Error;

      Uri::default()
    });

    Self {
      password,
      watchtower_uri,
      mode,
    }
  }
}
