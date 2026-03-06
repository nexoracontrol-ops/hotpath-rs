use std::thread;
#[hotpath::measure]
fn alloc() {
    for _ in 0..1000 {
        let vec = vec![1u8; 128];
        std::hint::black_box(vec);
    }
}

#[hotpath::main]
fn main() {
    let num_threads = std::env::var("HOTPATH_ALLOC_NUM_THREADS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3);
    let handles: Vec<_> = (0..num_threads)
        .map(|_| {
            thread::spawn(|| {
                for _ in 0..10_000 {
                    alloc();
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}
