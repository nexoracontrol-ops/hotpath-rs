use std::time::Instant;

#[inline(never)]
fn spin_1us() {
    let start = Instant::now();
    while start.elapsed().as_nanos() < 1000 {
        std::hint::spin_loop();
    }
}

#[hotpath::measure]
fn noop() {
    let a = 0;
    std::hint::black_box(a);
    spin_1us();
}

#[hotpath::main]
fn main() {
    let runs = std::env::var("HOTPATH_BENCH_RUNS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(100_000);

    let start = Instant::now();
    for _ in 0..runs {
        noop();
    }
    let elapsed = start.elapsed();

    println!(
        "noop: {runs} calls in {elapsed:?} ({:.1} ns/op)",
        elapsed.as_nanos() as f64 / runs as f64
    );
}
