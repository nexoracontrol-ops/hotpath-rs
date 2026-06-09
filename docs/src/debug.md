# Debug helpers and custom metrics 

`hotpath` provides macros for tracking values and logging debug info for profiled applications. All output is viewable in the TUI's `Debug` tab.

## `hotpath::dbg!`

Works like `std::dbg!` but sends debug output to the profiler instead of stderr. Logs are grouped by source location (file and line number) and viewable in the TUI.

```rust
#[hotpath::main]
fn main() {
    // Debug a single value - logs "3"
    hotpath::dbg!(1 + 2);

    // Debug multiple values
    hotpath::dbg!(foo(), bar());
}
```

## `hotpath::val!`

Tracks key-value pairs. Unlike `dbg!`, values are grouped by key name rather than source location, making it useful for tracking named metrics across different code locations.

```rust
#[hotpath::main]
fn main() {
    // Track a counter value
    let count = 42;
    hotpath::val!("request_count").set(&count);

    // Track state changes
    let state = "connected";
    hotpath::val!("connection_state").set(&state);
}
```

Values must implement `std::fmt::Debug`. Each call to `.set()` updates the entry and appends to its log history in the TUI.

## `hotpath::gauge!`

Tracks numeric values with set, increment, and decrement operations. Gauges display current value, and updates history in the TUI.

```rust
#[hotpath::main]
fn main() {
    // Set an absolute value
    hotpath::gauge!("queue_size").set(42.0);

    // Increment/decrement
    hotpath::gauge!("active_connections").inc(1.0);
    hotpath::gauge!("active_connections").dec(1.0);

    // Chain operations
    hotpath::gauge!("counter").set(0.0).inc(5.0).dec(2.0);
}
```
