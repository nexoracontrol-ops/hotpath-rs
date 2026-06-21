# Tokio Async Runtime Performance Metrics and Monitoring

`hotpath` monitors Tokio runtime performance by polling [`tokio::runtime::RuntimeMetrics`](https://docs.rs/tokio/latest/tokio/runtime/struct.RuntimeMetrics.html) on a dedicated background thread. This gives you visibility into `tokio-runtime-worker` thread utilization, task scheduling, and global/local queue depths without modifying your async code.

## Setup

Enable the `tokio` feature:

```toml
[dependencies]
hotpath = { version = "{{HOTPATH_VERSION}}", features = ["tokio"] }
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

## Reading Tokio RuntimeMetrics with `Handle::metrics()`

Under the hood Tokio exposes runtime stats through [`Handle::metrics()`](https://docs.rs/tokio/latest/tokio/runtime/struct.Handle.html#method.metrics). `hotpath` polls this same API for you, but you can read it directly:

```rust
let handle = tokio::runtime::Handle::current();
let metrics = handle.metrics();

let workers = metrics.num_workers();             // tokio-runtime-worker thread count
let alive = metrics.num_alive_tasks();           // currently alive tasks
let global_depth = metrics.global_queue_depth(); // tasks waiting in the global queue

println!("{workers} workers, {alive} alive tasks, {global_depth} queued");
```

Calling `Handle::current().metrics()` throughout application code adds boilerplate and spreads observability logic across request paths. `hotpath::tokio_runtime!()` samples these values on the `hp-runtime` thread instead and exposes them in the TUI and HTTP/MCP API.

## Monitoring Tokio worker threads

A multi-threaded runtime schedules tasks across a pool of `tokio-runtime-worker` threads (`worker_threads` on `runtime::Builder`, default = available CPU cores). Each worker drains its own local queue, steals from peers when idle, and falls back to the global injection queue. `hotpath` tracks per-worker behavior so you can see how evenly load spreads:

- **Worker utilization** - busy duration vs wall-clock time per worker. One worker pinned near 100% busy while others idle signals uneven scheduling or a blocking call hogging a thread.
- **Idle vs busy workers** - park count rises when a worker goes idle waiting for work; flat park counts under load mean the worker is saturated.
- **Work stealing** - steal count and steal operations show how often a worker pulls tasks from a peer's local queue. Heavy stealing means uneven local-queue distribution.
- **Queue depth** - growing local queue depth (per worker) or global queue depth (runtime-wide) means tasks arrive faster than workers drain them. Investigate blocking work, long polls, insufficient worker threads, or backpressure.

For a `current_thread` runtime the worker count is always `1` and steal counts stay `0`.

## Metrics collected

### Always available (stable Tokio API)

Global metrics:
- **Workers** - number of runtime worker threads, configured via `worker_threads` on `runtime::Builder`. Always `1` for `current_thread` runtime.
- **Alive tasks** - number of currently alive tasks. Increases when a task is spawned, decreases when a task exits.
- **Global queue depth** - current number of tasks pending in the global injection queue. Tasks spawned or notified from a non-runtime thread are scheduled here.

Per-worker metrics:
- **Park count** - total number of times the worker has parked (gone idle) waiting for new work. Monotonically increasing.
- **Busy duration** - cumulative time the worker has spent executing tasks. Monotonically increasing. High busy duration relative to wall-clock time indicates the worker is under load and will check for inbound events less often.

### With `tokio_unstable` (additional metrics)

Per-worker:
- **Poll count** - total number of task polls executed by this worker. Monotonically increasing.
- **Steal count** - total number of tasks this worker has stolen from other workers' queues. Only applies to the multi-threaded runtime (always `0` for `current_thread`). Monotonically increasing.
- **Steal operations** - number of times this worker successfully stole tasks from another worker. Each steal operation may transfer multiple tasks. Only applies to multi-threaded runtime. Monotonically increasing.
- **Overflow count** - number of times this worker's local queue was full, causing half of the local queue to be moved to the global injection queue. Only applies to multi-threaded runtime. Monotonically increasing.
- **Local queue depth** - current number of tasks pending in this worker's local queue. Tasks spawned or notified from within the runtime go here.
- **Mean poll time** - exponentially weighted moving average of task poll durations. Only provided by the multi-threaded runtime.

Global:
- **Blocking threads (total)** - number of additional threads spawned by `spawn_blocking`. Configured via `max_blocking_threads` on `runtime::Builder`.
- **Blocking threads (idle)** - number of idle blocking threads currently waiting for new `spawn_blocking` work.
- **Blocking queue depth** - current number of tasks pending in the blocking thread pool, waiting for a blocking thread to become available.
- **Spawned tasks count** - total number of tasks spawned since the runtime was created. Monotonically increasing.
- **Remote schedule count** - total number of tasks woken from outside the runtime. These tasks go through the global injection queue instead of a worker-local queue. Monotonically increasing.
- **IO driver FD registered count** - total number of file descriptors registered with the runtime's I/O driver. Requires `net` feature on Tokio.
- **IO driver FD deregistered count** - total number of file descriptors deregistered by the I/O driver. Current FD count = registered - deregistered. Requires `net` feature on Tokio.
- **IO driver ready count** - total number of I/O readiness events processed by the I/O driver. Requires `net` feature on Tokio.

To enable unstable metrics, build with:

```bash
RUSTFLAGS="--cfg tokio_unstable" cargo run --features='hotpath'
```

## Tokio RuntimeMetrics vs hotpath

`tokio::runtime::RuntimeMetrics` gives you runtime-level counters - worker count, queue depth, poll counts, busy duration. It tells you *that* the runtime is under load, not *which* code is responsible.

`hotpath` polls RuntimeMetrics on a background thread alongside application-level profiling, allowing you to correlate it with function timings, future poll counts and lifecycle, channel throughput, and mutex/lock contention. So instead of seeing only a deep global queue, you see which futures, channels, locks, or functions are causing worker saturation.

| | Tokio RuntimeMetrics | hotpath |
|---|---|---|
| Worker / queue / scheduling stats | Yes | Yes (polls RuntimeMetrics) |
| Per-function timing & allocations | No | Yes |
| Future / stream / channel profiling | No | Yes |
| Live TUI + HTTP/MCP access | No | Yes |

Use RuntimeMetrics directly when you only need raw runtime counters; use `hotpath` when you need to connect those counters to the application code causing the bottleneck.

## Configuration

- `HOTPATH_TOKIO_RUNTIME_INTERVAL_MS` - sampling interval in milliseconds (default: 1000)
