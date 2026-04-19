use std::time::{Duration, Instant};

const WAIT_FOR: Duration = Duration::from_micros(1);

// This function is used to spin-wait for a given duration.
// It's more precise than std::thread::sleep.
#[hotpath::measure]
fn spin_wait() {
    let start = Instant::now();
    while start.elapsed() < WAIT_FOR {}
}

#[hotpath::main]
fn main() {
    let runs = std::env::var("HOTPATH_BENCHMARK_NOOP_RUNS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(100_000);
    for _ in 0..runs {
        spin_wait();
    }
}
