# CPU profiling

In addition to execution time and allocations tracking, `hotpath` also supports CPU sampling. By comparing different profiling reports, you can determine whether a bottleneck is I/O-bound, CPU-bound, or memory-bound. See [Sampling comparison](/sampling_comparison) for a detailed explanation of how the profiling modes differ.

CPU profiling uses [samply](https://github.com/mstange/samply) and requires some additional system configuration.

## Configuring CPU profiling

Start by installing `samply`:

```bash
cargo install samply --locked
```

and verify version:

```bash
samply --version
```

`hotpath` is tested to work with version `0.13.x`.

Then install the `hotpath-samply` wrapper binary that ships with `hotpath`:

```bash
cargo install hotpath --bin hotpath-samply --version '^{{HOTPATH_VERSION}}'
```

Both `samply` and `hotpath-samply` must be available in your `PATH` at runtime. The host process spawns `hotpath-samply` as a child, which in turn invokes `samply` to record the profile.

### MacOS samply permissions

Run:

```bash
samply setup
```

It will prompt you to sign `samply` binary so that it can attach to a running process by its PID. 

### Linux samply permissions

On Linux, CPU profiling requires elevated kernel profiling permissions. Run:

```bash
echo -1 | sudo tee /proc/sys/kernel/perf_event_paranoid
echo 0 | sudo tee /proc/sys/kernel/yama/ptrace_scope
echo 0 | sudo tee /proc/sys/kernel/kptr_restrict
```

These settings allow `samply` to attach to a running process and collect kernel-level profiling information.

The configuration is temporary and resets after reboot. Consider persisting it via `/etc/sysctl.d/`.

Another caveat when using `samply` with `hotpath` on Linux is that you must prefix the profiled command with `setsid -w`. So instead of:

```bash
cargo run --features='hotpath,hotpath-alloc,hotpath-cpu'
```

you should run:

```bash
setsid -w cargo run --features='hotpath,hotpath-alloc,hotpath-cpu'
```

Otherwise the parent process may exit before `hotpath` finishes the profiling report.

## CPU profiling with hotpath

You must build with debug symbols enabled in order to attribute CPU samples to instrumented functions. Symbols are included by default in debug profile builds. `--release` builds don't include this info, so you should use a dedicated profile instead:

`Cargo.toml`
```toml
[profile.profiling]
inherits = "release"
debug = true
```

Now run your example with an additional `hotpath-cpu` flag: 

```bash
cargo run --features='hotpath,hotpath-alloc,hotpath-cpu' --profile profiling
```

it will output a CPU usage report (in addition to wall-clock time and allocations):

```
+------------------------+---------+---------+
| Function               | Samples | % Total |
+------------------------+---------+---------+
| cpu_basic::sync_work   | 1915914 | 56.13%  |
+------------------------+---------+---------+
| cpu_basic::async_sleep | 14056   | 0.41%   |
+------------------------+---------+---------+
| cpu_basic::sync_alloc  | 1581    | 0.05%   |
+------------------------+---------+---------+
samply load /tmp/hotpath/61089-1778083683167502000/hp.json.gz
```

You can optionally run the displayed `samply load` command to open an interactive performance report:

<img loading="lazy" src="{{#asset-hash images/samply-report.png}}" alt="Interactive samply performance report">

## Improving symbols attribution

Currently `hotpath` correctly attributes CPU usage to module functions instrumented with `measure` macro. `impl` functions instrumented with `measure` require an additional config:

```rust
impl Worker {
    #[hotpath::measure]
    fn run() {
      // ...
    }
}
```

This config won't correctly attribute CPU samples to the `run` function, because it's defined in an `impl` block. You have to explicitly declare `impl_type` to fix it:

```rust
impl Worker {
    #[hotpath::measure(impl_type = "Worker")]
    fn run() {
      // ...
    }
}
```

Alternatively you can use `measure_all` macro for `impl` functions to correctly attribute all symbols without additional config:

```rust
#[hotpath::measure_all]
impl Worker {
    fn run() {
      // ...
    }
}
```

## Current status

CPU profiling support is still experimental, so bug reports and feedback are highly appreciated.
