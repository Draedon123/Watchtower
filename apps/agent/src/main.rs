use std::{env, thread::sleep, time};

use http::{client::Client, header::Headers};

use watchtower_agent::{
  config::{Config, RunMode},
  help::display_help_message,
  system::{
    cpu_usage::CpuUsage,
    hostname::HostnameFetcher,
    memory::{Memory, MemoryType},
    register::loop_register,
  },
  test::spawn_instances,
};

#[tokio::main]
async fn main() {
  let args: Vec<String> = env::args().collect();
  let config = Config::new(&args);

  match config.mode {
    RunMode::Normal => {
      println!("Connecting to server on {}", config.watchtower_address);

      let mut client = Client::new();
      let headers = Headers::from(vec![
        ("Host", &config.watchtower_address as &str),
        ("Authorization", &config.password),
      ]);

      client.default_headers = headers;

      let hostname_fetcher = HostnameFetcher::new();
      loop_register(&config, &mut client, &hostname_fetcher).await;

      let mut cpu_usage = CpuUsage::new(&hostname_fetcher);
      let mut physical_memory = Memory::new(MemoryType::Physical, &hostname_fetcher);
      let mut swap_memory = Memory::new(MemoryType::Swap, &hostname_fetcher);
      loop {
        sleep(time::Duration::from_millis(1000));

        if let Err(error) = cpu_usage.send_to_watchtower(&config, &mut client).await {
          eprintln!("Failed to send cpu usage to watchtower: {error}");
        };

        if let Err(error) = physical_memory
          .send_to_watchtower(&config, &mut client)
          .await
        {
          eprintln!("Failed to send physical memory data to watchtower: {error}");
        };

        if let Err(error) = swap_memory.send_to_watchtower(&config, &mut client).await {
          eprintln!("Failed to send swap memory data to watchtower: {error}");
        };
      }
    }
    RunMode::Test(num_test_instances) => {
      println!("Spawning {num_test_instances} test instances");
      spawn_instances(num_test_instances).await;
    }
    RunMode::Help => {
      display_help_message();
    }
    RunMode::Unknown(mode) => {
      eprintln!("Unknown config mode \"{mode}\"");
    }
  };
}
