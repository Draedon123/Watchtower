use std::{env, thread::sleep, time};

use futures::future::join_all;
use http::{client::Client, header::Headers};
use rand::{RngExt, rngs::SmallRng};

use crate::{
  config::Config,
  system::{
    cpu_usage::CpuUsage,
    hostname::HostnameFetcher,
    memory::{Memory, MemoryData, MemoryType},
    register::loop_register,
  },
};

struct Instance {
  hostname_fetcher: HostnameFetcher,
  config: Config,
  client: Client,
  rng: SmallRng,

  cpu_usage: f32,
  max_swap: u64,
  swap_free: u64,
  max_physical: u64,
  physical_free: u64,
}

impl Instance {
  pub fn new(id: usize, config: Config) -> Result<Instance, String> {
    let mut hostname_fetcher = HostnameFetcher::new();
    let base_hostname = hostname_fetcher.get_hostname()?;
    let test_hostname = format!("{base_hostname} {id}");

    hostname_fetcher.set_override(Some(test_hostname));

    // TODO: reduce code repetition

    println!(
      "Instance {id} connecting to server on {}",
      config.watchtower_address
    );

    let mut client = Client::new();
    let headers = Headers::from(vec![
      ("Host", &config.watchtower_address as &str),
      ("Authorization", &config.password),
    ]);

    client.default_headers = headers;

    let possible_physical_memory_gib: Vec<u64> = vec![2, 4, 8, 16, 32, 64];
    let possible_swap_memory_gib: Vec<u64> = vec![2, 4, 8, 16];
    let mut rng: SmallRng = rand::make_rng();
    let max_physical_kib = 1024
      * 1024
      * possible_physical_memory_gib
        [rng.random::<u8>() as usize % possible_physical_memory_gib.len()];
    let max_swap_kib = 1024
      * 1024
      * possible_swap_memory_gib[rng.random::<u8>() as usize % possible_swap_memory_gib.len()];

    let cpu_usage: f32 = rng.random_range(0.3..0.7);
    let physical_free =
      rng.random_range(0.2 * (max_physical_kib as f32)..0.8 * (max_physical_kib as f32)) as u64;
    let swap_free =
      rng.random_range(0.2 * (max_swap_kib as f32)..0.8 * (max_swap_kib as f32)) as u64;

    let max_physical = max_physical_kib as u64;
    let max_swap = max_swap_kib as u64;

    Ok(Instance {
      hostname_fetcher,
      config,
      client,
      rng,
      cpu_usage,
      max_swap,
      swap_free,
      max_physical,
      physical_free,
    })
  }

  pub async fn register(&mut self) {
    loop_register(&self.config, &mut self.client, &self.hostname_fetcher).await;
  }

  pub async fn tick(&mut self) {
    self.send_data_to_watchtower().await;
    self.update_data();
  }

  async fn send_data_to_watchtower(&mut self) {
    if let Err(error) = CpuUsage::send_data_to_watchtower(
      &self.hostname_fetcher,
      self.cpu_usage,
      &self.config,
      &mut self.client,
    )
    .await
    {
      eprintln!("Failed to send cpu usage to watchtower: {error}");
    };

    let physical_memory = MemoryData {
      total_kib: self.max_physical,
      available_kib: self.physical_free,
    };

    if let Err(error) = Memory::send_data_to_watchtower(
      &self.hostname_fetcher,
      &MemoryType::Physical,
      Some(physical_memory),
      &self.config,
      &mut self.client,
    )
    .await
    {
      eprintln!("Failed to send physical memory data to watchtower: {error}");
    };

    let swap_memory = MemoryData {
      total_kib: self.max_swap,
      available_kib: self.swap_free,
    };
    if let Err(error) = Memory::send_data_to_watchtower(
      &self.hostname_fetcher,
      &MemoryType::Swap,
      Some(swap_memory),
      &self.config,
      &mut self.client,
    )
    .await
    {
      eprintln!("Failed to send swap memory data to watchtower: {error}");
    };
  }

  fn update_data(&mut self) {
    self.cpu_usage = f32::clamp(
      self.cpu_usage * self.rng.random_range((1.0 / 1.1)..1.1),
      0.0,
      1.0,
    );
    self.physical_free = u64::clamp(
      (self.physical_free as f32 * self.rng.random_range((1.0 / 1.1)..1.1)) as u64,
      0,
      self.max_physical,
    );
    self.swap_free = u64::clamp(
      (self.swap_free as f32 * self.rng.random_range((1.0 / 1.1)..1.1)) as u64,
      0,
      self.max_swap,
    );
  }
}

pub async fn spawn_instances(num_instances: usize) {
  let args: Vec<String> = env::args().collect();
  let config = Config::new(&args);

  let instances: &mut Vec<Instance> = Box::leak(Box::new(Vec::with_capacity(num_instances)));

  for id in 1..=num_instances {
    let instance = Instance::new(id, config.clone());

    match instance {
      Ok(mut instance) => {
        instance.register().await;
        instances.push(instance)
      }
      Err(error) => eprintln!("Failed to spawn test instance {id}: {error}"),
    }
  }

  loop {
    let mut futures = Vec::with_capacity(num_instances);

    for instance in &mut *instances {
      futures.push(instance.tick());
    }

    join_all(futures).await;

    sleep(time::Duration::from_millis(1000));
  }
}
