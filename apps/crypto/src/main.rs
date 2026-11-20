use std::time::Instant;

use crypto::hash::sha1;

fn main() {
  let num_trials = 16;
  let mut total_elapsed = 0.0;

  for _ in 0..num_trials {
    let test_128_mib = String::from(
      "abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmno",
    )
    .repeat(2usize.pow(21))
    .into_bytes();

    let start = Instant::now();
    let _ = sha1(test_128_mib);
    let elapsed = Instant::elapsed(&start).as_secs_f64();
    let throughput_mib_s = 128.0 / elapsed;

    total_elapsed += elapsed;

    println!(
      "128MiB SHA1: {:.4}s ({:.2} MiB/s)",
      elapsed, throughput_mib_s
    );
  }

  println!(
    "Average: {:.4}s ({:.2} MiB/s)",
    total_elapsed / num_trials as f64,
    128.0 * num_trials as f64 / total_elapsed
  );
}
