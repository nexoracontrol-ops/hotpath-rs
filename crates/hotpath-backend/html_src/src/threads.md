# Thread performance monitoring: CPU and memory metrics per thread

It's enabled by default via `threads` feature flag. The monitoring dashboard displays real-time performance metrics for all active and sleeping threads in the instrumented process. For each thread, you can observe:

- PID – Process identifier
- Total Alloc – Dealloc – Aggregate allocation delta for all threads
- RSS – Resident Set Size (total physical memory currently used)

and per-thread metrics:

- Thread Name – Logical name 
- TID – System thread identifier
- Status – Current execution state (e.g., `Sleeping`, `Running`, `Blocked`, etc.)
- CPU % – Instant CPU utilization per thread
- User / Sys Time – Cumulative user-mode and kernel-mode execution time
- Alloc / Dealloc – Total allocated and deallocated memory attributed to that thread
- Diff – Net allocation difference (growth or shrinkage in allocated memory since last refresh)

<img loading="lazy" src="{{#asset-hash images/threads-view.png}}" alt="hotpath-rs TUI showing per-thread CPU and memory usage monitoring">
