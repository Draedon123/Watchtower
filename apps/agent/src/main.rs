use std::{env, thread::sleep, time};

use http::{client::Client, header::Headers};

use watchtower_agent::{
  config::{Config, RunMode},
  help::display_help_message,
  system::{
    cpu_usage::CpuUsage, hostname::get_hostname, memory::MemoryType, register::loop_register,
  },
  test::spawn_instances,
  watchtower_bridge::{Bridge, send_to_watchtower},
};

#[tokio::main]
async fn main() {
  let args: Vec<String> = env::args().collect();
  let config = Config::new(&args);
  let hostname = match get_hostname() {
    Ok(hostname) => hostname,
    Err(error) => {
      eprintln!("{error}");
      return;
    }
  };

  match config.mode {
    RunMode::Normal => {
      println!("Connecting to server on {}", config.watchtower_address);

      let mut client = Client::new();
      let headers = Headers::from(vec![
        ("Host", &config.watchtower_address as &str),
        ("Authorization", &config.password),
      ]);

      client.default_headers = headers;

      loop_register(&config, &mut client, &hostname).await;

      let cpu_usage = CpuUsage::new();
      let physical_memory = MemoryType::Physical;
      let swap_memory = MemoryType::Swap;

      let mut statistics: Vec<Box<dyn Bridge>> = vec![
        Box::new(cpu_usage),
        Box::new(physical_memory),
        Box::new(swap_memory),
      ];

      loop {
        sleep(time::Duration::from_millis(1000));

        if let Err(error) =
          send_to_watchtower(&mut statistics, &mut client, &config, &hostname).await
        {
          eprintln!("{error}");
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
