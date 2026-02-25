# Tokio runtime performance monitoring: worker stats and task metrics

`hotpath` can monitor Tokio runtime performance by polling [`tokio::runtime::RuntimeMetrics`](https://docs.rs/tokio/latest/tokio/runtime/struct.RuntimeMetrics.html) on a dedicated background thread. This gives you visibility into worker thread utilization, task scheduling metrics, and queue depths without modifying your async code.

## Setup

Enable the `tokio` feature:

```toml
[dependencies]
hotpath = { version = "0.13", features = ["tokio"] }
```

Then call `tokio_runtime!()` inside your async main:

```rust
#[tokio::main]
#[hotpath::main]
async fn main() {
    hotpath::tokio_runtime!();

    // ...
}
```

You can also pass an explicit handle if you're not inside a Tokio context:

```rust
let handle = tokio::runtime::Handle::current();
hotpath::tokio_runtime!(&handle);
```

The macro spawns a dedicated `hp-runtime` background thread that periodically samples runtime metrics and stores snapshots for the TUI and HTTP API.

## Metrics collected

### Always available (stable Tokio API)

Global metrics:
- **Workers** — number of runtime worker threads, configured via `worker_threads` on `runtime::Builder`. Always `1` for `current_thread` runtime.
- **Alive tasks** — number of currently alive tasks. Increases when a task is spawned, decreases when a task exits.
- **Global queue depth** — current number of tasks pending in the global injection queue. Tasks spawned or notified from a non-runtime thread are scheduled here.

Per-worker metrics:
- **Park count** — total number of times the worker has parked (gone idle) waiting for new work. Monotonically increasing.
- **Busy duration** — cumulative time the worker has spent executing tasks. Monotonically increasing. High busy duration relative to wall-clock time indicates the worker is under load and will check for inbound events less often.

### With `tokio_unstable` (additional metrics)

Per-worker:
- **Poll count** — total number of task polls executed by this worker. Monotonically increasing.
- **Steal count** — total number of tasks this worker has stolen from other workers' queues. Only applies to the multi-threaded runtime (always `0` for `current_thread`). Monotonically increasing.
- **Steal operations** — number of times this worker successfully stole tasks from another worker. Each steal operation may transfer multiple tasks. Only applies to multi-threaded runtime. Monotonically increasing.
- **Overflow count** — number of times this worker's local queue was full, causing half of the local queue to be moved to the global injection queue. Only applies to multi-threaded runtime. Monotonically increasing.
- **Local queue depth** — current number of tasks pending in this worker's local queue. Tasks spawned or notified from within the runtime go here.
- **Mean poll time** — exponentially weighted moving average of task poll durations. Only provided by the multi-threaded runtime.

Global:
- **Blocking threads (total)** — number of additional threads spawned by `spawn_blocking`. Configured via `max_blocking_threads` on `runtime::Builder`.
- **Blocking threads (idle)** — number of idle blocking threads currently waiting for new `spawn_blocking` work.
- **Blocking queue depth** — current number of tasks pending in the blocking thread pool, waiting for a blocking thread to become available.
- **Spawned tasks count** — total number of tasks spawned since the runtime was created. Monotonically increasing.
- **Remote schedule count** — total number of tasks woken from outside the runtime. These tasks must go through the global injection queue, which tends to be slower than local scheduling. Monotonically increasing.
- **IO driver FD registered count** — total number of file descriptors registered with the runtime's I/O driver. Requires `net` feature on Tokio.
- **IO driver FD deregistered count** — total number of file descriptors deregistered by the I/O driver. Current FD count = registered - deregistered. Requires `net` feature on Tokio.
- **IO driver ready count** — total number of I/O readiness events processed by the I/O driver. Requires `net` feature on Tokio.

To enable unstable metrics, build with:

```bash
RUSTFLAGS="--cfg tokio_unstable" cargo run --features='hotpath'
```

## Config 

- `HOTPATH_TOKIO_RUNTIME_INTERVAL_MS` - sampling interval in milliseconds (default: 1000)
