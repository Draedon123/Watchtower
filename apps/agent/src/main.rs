use std::{env, thread::sleep, time};

use crate::{config::Config, register::loop_register, system::cpu_usage::CpuUsage};

mod config;
mod register;
mod system;

fn main() {
  let args: Vec<String> = env::args().collect();
  let config = Config::new(&args);

  loop_register(&config);

  let mut cpu_usage = CpuUsage::new();
  loop {
    sleep(time::Duration::from_millis(1000));

    cpu_usage.send_to_watchtower(&config);
  }
}
