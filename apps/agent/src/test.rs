use std::{env, thread::sleep, time};

use futures::future::join_all;
use http::{client::Client, header::Headers};
use rand::{RngExt, rngs::SmallRng};
use watchtower_server::data::DataEntry;

use crate::{
  config::Config,
  system::{hostname::get_hostname, memory::MemoryType, register::loop_register},
  watchtower_bridge::{Bridge, send_to_watchtower},
};

trait Testable: Updatable + Bridge {}
pub trait Updatable {
  fn update(&mut self);
}

pub struct TestCpuUsage {
  cpu_usage: f32,
  rng: SmallRng,
}

impl TestCpuUsage {
  pub fn new() -> TestCpuUsage {
    let mut rng: SmallRng = rand::make_rng();
    let cpu_usage: f32 = rng.random_range(0.3..0.7);

    TestCpuUsage { cpu_usage, rng }
  }
}

impl Testable for TestCpuUsage {}

impl Bridge for TestCpuUsage {
  fn get_data(&mut self) -> Result<(String, DataEntry), String> {
    Ok((String::from("cpu_usage"), DataEntry::from(self.cpu_usage)))
  }
}

impl Updatable for TestCpuUsage {
  fn update(&mut self) {
    self.cpu_usage = f32::clamp(
      self.cpu_usage * self.rng.random_range((1.0 / 1.1)..1.1),
      0.0,
      1.0,
    );
  }
}

pub struct TestMemoryUsage {
  memory_type: MemoryType,
  max: u64,
  free: u64,
  rng: SmallRng,
}

impl TestMemoryUsage {
  pub fn new(memory_type: MemoryType) -> TestMemoryUsage {
    let mut rng: SmallRng = rand::make_rng();
    let possible_max_gib: Vec<u64> = match memory_type {
      MemoryType::Physical => vec![2, 4, 8, 16, 32, 64],
      MemoryType::Swap => vec![2, 4, 8, 16],
    };

    let max_kib =
      1024 * 1024 * possible_max_gib[rng.random::<u8>() as usize % possible_max_gib.len()];
    let free_kib = rng.random_range(0.2 * (max_kib as f32)..0.8 * (max_kib as f32)) as u64;

    TestMemoryUsage {
      memory_type,
      max: max_kib,
      free: free_kib,
      rng,
    }
  }
}

impl Testable for TestMemoryUsage {}

impl Updatable for TestMemoryUsage {
  fn update(&mut self) {
    self.free = u64::clamp(
      (self.free as f32 * self.rng.random_range((1.0 / 1.1)..1.1)) as u64,
      0,
      self.max,
    );
  }
}

impl Bridge for TestMemoryUsage {
  fn get_data(&mut self) -> Result<(String, DataEntry), String> {
    Ok((
      self.memory_type.watchtower_path().to_string(),
      DataEntry::from((self.free, self.max)),
    ))
  }
}

struct Instance {
  hostname: String,
  config: Config,
  client: Client,
  test_stats: Vec<Box<dyn Testable>>,
}

impl Instance {
  pub fn new(id: usize, config: Config) -> Result<Instance, String> {
    let hostname = get_hostname()?;
    let hostname = format!("{hostname} {id}");
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

    let test_stats: Vec<Box<dyn Testable>> = vec![
      Box::new(TestCpuUsage::new()),
      Box::new(TestMemoryUsage::new(MemoryType::Physical)),
      Box::new(TestMemoryUsage::new(MemoryType::Swap)),
    ];

    Ok(Instance {
      hostname,
      config,
      client,
      test_stats,
    })
  }

  pub async fn register(&mut self) {
    loop_register(&self.config, &mut self.client, &self.hostname).await;
  }

  pub async fn tick(&mut self) {
    self.send_data_to_watchtower().await;
    self.update_data();
  }

  pub fn update_data(&mut self) {
    for client in &mut self.test_stats {
      client.update();
    }
  }

  async fn send_data_to_watchtower(&mut self) {
    if let Err(error) = send_to_watchtower(
      &mut self.test_stats,
      &mut self.client,
      &self.config,
      &self.hostname,
    )
    .await
    {
      eprintln!(
        "{} failed to send data to watchtower: {error}",
        self.hostname
      );

      match self.client.try_reconnect_once().await {
        Ok(_) => println!("{} reconnected", self.hostname),
        Err(error) => eprintln!("{} failed to reconnect: {error}", self.hostname),
      };
    };
  }
}

pub async fn spawn_instances(num_instances: usize) {
  let args: Vec<String> = env::args().collect();
  let config = Config::new(&args);

  let mut instances: Vec<Instance> = Vec::with_capacity(num_instances);

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
