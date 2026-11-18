use std::collections::{HashMap, HashSet};

pub fn parse_args(
  args: &[String],
) -> (HashMap<String, String>, HashSet<String>, Vec<String>) {
  let mut config_args = HashMap::<String, String>::new();
  let mut flags = HashSet::<String>::new();
  let mut rest = Vec::<String>::new();

  let mut i: usize = 0;
  while i < args.len() {
    let arg = &args[i];

    if let Some(key) = arg.strip_prefix("--") {
      let value_index = i + 1;

      if value_index >= args.len() {
        eprintln!("Argument \"{arg}\" is missing value");
        break;
      }

      let value = args[value_index].clone();
      i += 1;

      config_args.insert(key.to_owned(), value);
    } else if let Some(flag) = arg.strip_prefix("-") {
      flags.insert(flag.to_owned());
    } else {
      rest.push(arg.to_owned());
    }

    i += 1;
  }

  (config_args, flags, rest)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn full_parse() {
    let args = [
      String::from("--foo"),
      String::from("bar"),
      String::from("-flag"),
      String::from("baz"),
      String::from("--invalid"),
    ];

    let (config_args, flags, rest) = parse_args(&args);

    assert_eq!(config_args.get("foo"), Some(&String::from("bar")));
    assert!(flags.contains("flag"));
    assert!(rest.contains(&String::from("baz")));
    assert_eq!(config_args.len(), 1);
  }
}
