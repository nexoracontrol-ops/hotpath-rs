# Profiling modes

`hotpath` supports two complementary approaches to performance monitoring.

## Static reports

Best for CLI tools, tests, or short-lived applications. On exit, `hotpath` prints a summary of execution time, memory usage, and timing percentiles. Reports can be rendered as readable tables or exported as JSON for automated analysis.

Every instrumented program prints a performance report automatically when executed with the `hotpath` feature enabled.

```bash
cargo run --features=hotpath
```

<img src="{{#asset-hash images/hotpath-timing-report.png}}" alt="hotpath-rs timing profiling report showing per-function execution statistics">

Use `--features='hotpath,hotpath-alloc'` to print memory usage report:

```bash
cargo run --features='hotpath,hotpath-alloc'
```

<img src="{{#asset-hash images/hotpath-alloc-report.png}}" alt="hotpath-rs memory allocation profiling report showing per-function byte counts">

Enable JSON output by setting `HOTPATH_JSON=true`.

## Live TUI dashboard 

Best for long-running processes like HTTP servers, or background workers. It continuously displays functions performance, allocation counters, and channel/stream throughput while the application is running. This mode helps diagnose runtime bottlenecks, queue buildup, and data flow issues that are not visible in static summaries.

Install the TUI with:

```
cargo install hotpath --features=tui
```

Run the dashboard:

```
hotpath console
```

Then launch your instrumented application (with `hotpath` feature enabled) in a separate terminal to see live performance metrics.

<video width="100%" loop muted playsinline controls poster="{{#asset-hash images/hotpath-live-dashboard-poster.jpg}}">
  <source src="{{#asset-hash videos/hotpath-live-dashboard.mp4}}" type="video/mp4">
</video>

You can learn how to instrument any Rust program in the next sections.
