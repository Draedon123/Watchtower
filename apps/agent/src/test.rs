use std::{
  env,
  thread::{self, JoinHandle, sleep},
  time,
};

use http::{client::Client, header::Headers};
use rand::RngExt;

use crate::{
  config::Config,
  system::{
    cpu_usage::CpuUsage,
    hostname::HostnameFetcher,
    memory::{Memory, MemoryData, MemoryType},
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
  // TODO: fix http library to allow names with spaces
  let test_hostname = format!("{base_hostname}_{id}");

  hostname_fetcher.set_override(Some(test_hostname));

  // TODO: reduce code repetition
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

    let possible_physical_memory_gib: Vec<u64> = vec![2, 4, 8, 16, 32, 64];
    let possible_swap_memory_gib: Vec<u64> = vec![2, 4, 8, 16];
    let mut rng = rand::rng();
    let max_physical_kib = 1024
      * 1024
      * possible_physical_memory_gib
        [rng.random::<u8>() as usize % possible_physical_memory_gib.len()];
    let max_swap_kib = 1024
      * 1024
      * possible_swap_memory_gib[rng.random::<u8>() as usize % possible_swap_memory_gib.len()];

    let mut cpu_usage: f32 = rng.random_range(0.0..0.5);
    let mut physical_free =
      rng.random_range(0.05 * (max_physical_kib as f32)..0.8 * (max_physical_kib as f32)) as u64;
    let mut swap_free =
      rng.random_range(0.05 * (max_swap_kib as f32)..0.8 * (max_swap_kib as f32)) as u64;

    let max_physical = max_physical_kib as u64;
    let max_swap = max_swap_kib as u64;

    loop {
      sleep(time::Duration::from_millis(1000));

      if let Err(error) =
        CpuUsage::send_data_to_watchtower(&hostname_fetcher, cpu_usage, &config, &mut client)
      {
        eprintln!("Failed to send cpu usage to watchtower: {error}");
      };

      let physical_memory = MemoryData {
        total_kib: max_physical,
        available_kib: physical_free,
      };
      if let Err(error) = Memory::send_data_to_watchtower(
        &hostname_fetcher,
        &MemoryType::Physical,
        Some(physical_memory),
        &config,
        &mut client,
      ) {
        eprintln!("Failed to send physical memory data to watchtower: {error}");
      };

      let swap_memory = MemoryData {
        total_kib: max_swap,
        available_kib: swap_free,
      };
      if let Err(error) = Memory::send_data_to_watchtower(
        &hostname_fetcher,
        &MemoryType::Swap,
        Some(swap_memory),
        &config,
        &mut client,
      ) {
        eprintln!("Failed to send swap memory data to watchtower: {error}");
      };

      cpu_usage = f32::clamp(cpu_usage * rng.random_range(0.9..1.1), 0.0, 1.0);
      physical_free = u64::clamp(
        (physical_free as f32 * rng.random_range(0.9..1.1)) as u64,
        0,
        max_physical,
      );
      swap_free = u64::clamp(
        (swap_free as f32 * rng.random_range(0.9..1.1)) as u64,
        0,
        max_swap,
      );
    }
  }))
}
