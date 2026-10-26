use std::{env, thread::sleep, time};

use http::{client::Client, header::Headers};

use watchtower_agent::{
  config::Config,
  system::{
    cpu_usage::CpuUsage,
    memory::{Memory, MemoryType},
    register::loop_register,
  },
};

fn main() {
  let args: Vec<String> = env::args().collect();
  let config = Config::new(&args);

  let mut client = Client::new();
  let mut headers = Headers::new();
  headers
    .set_header("Host", &config.watchtower_address)
    .set_header("Authorization", &config.password);

  client.default_headers = headers;

  loop_register(&config, &mut client);

  let mut cpu_usage = CpuUsage::new();
  let mut physical_memory = Memory::new(MemoryType::Physical);
  let mut swap_memory = Memory::new(MemoryType::Swap);
  loop {
    sleep(time::Duration::from_millis(1000));

    if let Err(error) = cpu_usage.send_to_watchtower(&config, &mut client) {
      eprintln!("Failed to send cpu usage to watchtower: {error}");
    };

    if let Err(error) = physical_memory.send_to_watchtower(&config, &mut client) {
      eprintln!("Failed to send physical memory data to watchtower: {error}");
    };

    if let Err(error) = swap_memory.send_to_watchtower(&config, &mut client) {
      eprintln!("Failed to send swap memory data to watchtower: {error}");
    };
  }
}
