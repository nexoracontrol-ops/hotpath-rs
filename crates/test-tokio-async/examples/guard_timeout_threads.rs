use std::time::Duration;

fn main() {
    hotpath::threads::ThreadsGuardBuilder::new().build_with_timeout(Duration::from_secs(2));

    loop {
        std::hint::black_box(0u64.wrapping_add(1));
        std::thread::sleep(Duration::from_millis(10));
    }
}
