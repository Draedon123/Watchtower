use std::{env, thread::sleep, time};

use crate::{
  config::Config,
  system::{cpu_usage::CpuUsage, register::loop_register},
};

mod config;
mod system;

fn main() {
  let args: Vec<String> = env::args().collect();
  let config = Config::new(&args);

  loop_register(&config);

  let mut cpu_usage = CpuUsage::new();
  loop {
    sleep(time::Duration::from_millis(1000));

    if let Err(_) = cpu_usage.send_to_watchtower(&config) {
      eprintln!("Failed to send cpu usage to watchtower");
    };
  }
}
