# Rust Performance & Memory Profiler

<div class="hero-row">
  <img src="{{#asset-hash images/hotpath-ferris.webp}}" alt="hotpath-rs Rust profiler mascot Ferris the crab" class="ferris-img-hero">
  <div class="ssh-demo-container">
    <p class="ssh-demo-label">Try the TUI demo via SSH - no installation required:</p>
    <div class="terminal-shell">
      <span class="terminal-prompt">$</span>
      <span class="terminal-command">ssh demo.hotpath.rs</span>
    </div>
  </div>
</div>

[hotpath-rs](https://github.com/pawurb/hotpath-rs) is a Rust performance profiler that instruments functions, channels, futures, and streams. It helps you find runtime bottlenecks and optimize where it matters most, with detailed metrics for time, memory, and async data flow monitoring.
<div style="clear: both;"></div>

<div class="trusted-by">
  <p class="trusted-by-tagline">Trusted by dozens of open-source projects, including:</p>
  <div class="trusted-by-grid">
    <a href="https://github.com/apache/opendal" target="_blank" class="trusted-by-project">
      <span class="trusted-by-name">apache/opendal</span>
      <img src="{{#asset-hash images/stars-apache-opendal.svg}}" alt="opendal GitHub stars">
    </a>
    <a href="https://github.com/apache/horaedb" target="_blank" class="trusted-by-project">
      <span class="trusted-by-name">apache/horaedb</span>
      <img src="{{#asset-hash images/stars-apache-horaedb.svg}}" alt="horaedb GitHub stars">
    </a>
    <a href="https://github.com/marc2332/freya" target="_blank" class="trusted-by-project">
      <span class="trusted-by-name">marc2332/freya</span>
      <img src="{{#asset-hash images/stars-marc2332-freya.svg}}" alt="freya GitHub stars">
    </a>
    <a href="https://github.com/tqwewe/kameo" target="_blank" class="trusted-by-project">
      <span class="trusted-by-name">tqwewe/kameo</span>
      <img src="{{#asset-hash images/stars-tqwewe-kameo.svg}}" alt="kameo GitHub stars">
    </a>
    <a href="https://github.com/tryandromeda/andromeda" target="_blank" class="trusted-by-project">
      <span class="trusted-by-name">tryandromeda/andromeda</span>
      <img src="{{#asset-hash images/stars-tryandromeda-andromeda.svg}}" alt="andromeda GitHub stars">
    </a>
    <a href="https://github.com/nyudenkov/pysentry" target="_blank" class="trusted-by-project">
      <span class="trusted-by-name">nyudenkov/pysentry</span>
      <img src="{{#asset-hash images/stars-nyudenkov-pysentry.svg}}" alt="pysentry GitHub stars">
    </a>
  </div>
</div>

You can use it to produce one-off performance (timing or memory) reports:

<img loading="lazy" src="{{#asset-hash images/hotpath-alloc-report.png}}" alt="hotpath-rs memory allocation profiling report showing per-function byte counts">

or use the live TUI dashboard to monitor real-time performance and data flow metrics with debug info:

<video loading="lazy" width="100%" loop muted playsinline controls poster="{{#asset-hash images/hotpath-live-dashboard-poster.jpg}}">
  <source src="{{#asset-hash videos/hotpath-live-dashboard.mp4}}" type="video/mp4">
</video>

## Features

- **Zero-cost when disabled** - fully gated by a feature flag.
- **Low-overhead** profiling for both sync and async code.
- **Live TUI dashboard** - real-time monitoring of performance data flow metrics in TUI dashboard (built with <a href="https://ratatui.rs/" target="_blank">ratatui.rs</a>).
- **Static reports for one-off programs** - alternatively print profiling summaries without running the TUI.
- **Memory allocation tracking** - track bytes allocated and allocation counts per function.
- **Channel and stream monitoring** - instrument channels and streams to track message flow and throughput.
- **Futures instrumentation** - monitor any async piece of code to track poll counts, lifecycle and resolved values.
- **Detailed stats**: avg, total time, call count, % of total runtime, and configurable percentiles (p95, p99, etc.).
- **Tokio runtime monitoring** - track worker thread utilization, task scheduling, and queue depths.
- **MCP server for AI agents** - built-in <a href="https://modelcontextprotocol.io/" target="_blank">Model Context Protocol</a> server that lets LLMs query profiling data in real-time.
- **GitHub Actions integration** - configure CI to automatically benchmark your program against a base branch for each PR.

## Quick demo

Other then the SSH demo an easy way to quickly try the TUI is to run it in **auto-instrumentation mode**. The TUI process profiles itself and displays its own performance metrics in real time.

First, install `hotpath` CLI with auto-instrumentation enabled:

```bash
cargo install hotpath --features='tui,hotpath,hotpath-alloc'
```

Then launch the console:

```bash
hotpath console
```

and you'll see timing, memory and channel usage metrics.

Make sure to reinstall it without the auto-profiling features so that you can also observe metrics of other programs!

```bash
cargo install hotpath --features='tui'
```

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
hotpath = "0.10"

[features]
hotpath = ["hotpath/hotpath"]
hotpath-alloc = ["hotpath/hotpath-alloc"]
```

This config ensures that the lib has no compile time or runtime overhead unless explicitly enabled via a `hotpath` feature. All the lib dependencies are optional (i.e. not compiled) and all macros are noop unless profiling is enabled.

## Learn more

See the rest of the docs to learn how to instrument and profile your program:

- [Sampling Comparison](./sampling_comparison.html) - when to use `hotpath` vs CPU sampling profilers
- [Profiling modes](./profiling_modes.html) - static reports vs live TUI dashboard
- [Functions](./functions.html) - measure execution time and memory allocations
- [Futures](./futures.html) - monitor async code, poll counts, and resolved values
- [Channels](./channels.html) - track messages flow and throughput
- [Streams](./streams.html) - instrument async streams
- [Threads](./threads.html) - monitor threads usage
- [Tokio Runtime](./tokio_runtime.html) - monitor Tokio runtime worker stats and task scheduling
- [MCP Server](./mcp.html) - LLM integration via Model Context Protocol
