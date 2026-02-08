# Channel performance monitoring: message flow and throughput metrics

## hotpath::channel! macro 

This macro wraps channel creation to automatically track performance metrics and data flow:

```rust
use tokio::sync::mpsc;

#[tokio::main]
#[hotpath::main]
async fn main() {
    // Create and instrument a channel in one step
    let (tx, mut rx) = hotpath::channel!(mpsc::channel::<String>(100));

    // Use the channel exactly as before
    tx.send("Hello".to_string()).await.unwrap();
    let msg = rx.recv().await.unwrap();
}
```

[std::sync](https://doc.rust-lang.org/stable/std/sync/mpsc/index.html) channels can be instrumented by default. Enable `tokio`, `futures`, or `crossbeam` features for [Tokio](https://github.com/tokio-rs/tokio), [futures-rs](https://github.com/rust-lang/futures-rs), and [crossbeam](https://github.com/crossbeam-rs/crossbeam) channels, respectively.

**Supported channel types:**
- [`tokio::sync::mpsc::channel`](https://docs.rs/tokio/latest/tokio/sync/mpsc/fn.channel.html)
- [`tokio::sync::mpsc::unbounded_channel`](https://docs.rs/tokio/latest/tokio/sync/mpsc/fn.unbounded_channel.html)
- [`tokio::sync::oneshot::channel`](https://docs.rs/tokio/latest/tokio/sync/oneshot/fn.channel.html)
- [`futures_channel::mpsc::channel`](https://docs.rs/futures-channel/latest/futures_channel/mpsc/fn.channel.html)
- [`futures_channel::mpsc::unbounded`](https://docs.rs/futures-channel/latest/futures_channel/mpsc/fn.unbounded.html)
- [`futures_channel::oneshot::channel`](https://docs.rs/futures-channel/latest/futures_channel/oneshot/fn.channel.html)
- [`crossbeam_channel::bounded`](https://docs.rs/crossbeam/latest/crossbeam/channel/fn.bounded.html)
- [`crossbeam_channel::unbounded`](https://docs.rs/crossbeam/latest/crossbeam/channel/fn.unbounded.html)


## Optional config

```rust
// Custom label for easier identification in TUI
let (tx, rx) = hotpath::channel!(mpsc::channel::<String>(100), label = "worker_queue");

// Enable message logging (requires std::fmt::Debug trait on message type)
let (tx, rx) = hotpath::channel!(mpsc::channel::<String>(100), log = true);
```

Label channels to display them on top of the list. By passing `log = true` TUI will display messages that a channel received.

<img src="{{#asset-hash images/channels-log.png}}" alt="hotpath-rs TUI showing channel message flow monitoring with send and receive logs">

## Capacity parameter requirement

For `futures::channel::mpsc` bounded channels, you **must** specify the `capacity` parameter because their API doesn't expose the capacity after creation:

```rust
use futures_channel::mpsc;

// futures bounded channel - MUST specify capacity
let (tx, rx) = hotpath::channel!(mpsc::channel::<String>(10), capacity = 10);
```

Tokio and crossbeam channels don't require this parameter because their capacity is accessible from the channel handles.

## A note on accuracy

`hotpath` instruments channels by using a proxy on the receive side with the capacity of 1. Messages flow directly into your original channel, then through a proxy before reaching the consumer. This design adds 1 slot of extra buffering for bounded channels.

Please note that enabling monitoring can subtly affect channel behavior in some cases. For example, using `try_send` may behave slightly differently since the proxy adds 1 slot of extra capacity. Also some wrappers currently not propagate info about receiver getting dropped.

I'm actively improving the library, so any feedback, issues, bug reports are appreciated.
