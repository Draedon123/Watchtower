use std::{
  env,
  thread::{self, JoinHandle, sleep},
  time,
};

use http::{client::Client, header::Headers};

use crate::{
  config::Config,
  system::{
    cpu_usage::CpuUsage,
    hostname::HostnameFetcher,
    memory::{Memory, MemoryType},
    register::loop_register,
  },
};

pub fn spawn_instances(num_instances: usize) {
  let mut threads: Vec<JoinHandle<()>> = Vec::with_capacity(num_instances);

  for id in 1..=num_instances {
    match spawn_instance(id) {
      Ok(thread) => {
        println!("Spawning instance {id}");
        threads.push(thread)
      }
      Err(error) => eprintln!("Failed to spawn test instance {id}: {error}"),
    }
  }

  for thread in threads {
    thread.join().unwrap();
  }
}

fn spawn_instance(id: usize) -> Result<JoinHandle<()>, String> {
  let mut hostname_fetcher = HostnameFetcher::new();
  let base_hostname = hostname_fetcher.get_hostname()?;
  let test_hostname = format!("{base_hostname}_{id}");

  hostname_fetcher.set_override(Some(test_hostname));

  Ok(thread::spawn(move || {
    let args: Vec<String> = env::args().collect();
    let config = Config::new(&args);

    let mut client = Client::new();
    let mut headers = Headers::new();
    headers
      .set_header("Host", &config.watchtower_address)
      .set_header("Authorization", &config.password);

    client.default_headers = headers;

    loop_register(&config, &mut client, &hostname_fetcher);

    let mut cpu_usage = CpuUsage::new(&hostname_fetcher);
    let mut physical_memory = Memory::new(MemoryType::Physical, &hostname_fetcher);
    let mut swap_memory = Memory::new(MemoryType::Swap, &hostname_fetcher);
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
  }))
}
