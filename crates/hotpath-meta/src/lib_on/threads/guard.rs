use std::io::Write;
use std::path::PathBuf;
use std::thread;
use std::time::Instant;

use prettytable::{Cell, Row, Table};

use crate::output::resolve_output_path;
use crate::threads::{get_threads_json, init_threads_monitoring};
use crate::Format;

#[must_use = "builder is discarded without creating a guard"]
pub struct ThreadsGuardBuilder {
    format: Format,
    output_path: Option<PathBuf>,
}

impl ThreadsGuardBuilder {
    pub fn new() -> Self {
        Self {
            format: Format::default(),
            output_path: None,
        }
    }

    pub fn format(mut self, format: Format) -> Self {
        self.format = format;
        self
    }

    pub fn output_path(mut self, path: impl AsRef<std::path::Path>) -> Self {
        self.output_path = Some(resolve_output_path(path));
        self
    }

    pub fn build(self) -> ThreadsGuard {
        init_threads_monitoring();
        #[cfg(feature = "hotpath-alloc-meta")]
        crate::functions::alloc::core::init_thread_alloc_tracking();
        ThreadsGuard {
            start_time: Instant::now(),
            format: self.format,
            output_path: self.output_path,
        }
    }

    pub fn build_with_timeout(self, duration: std::time::Duration) {
        let guard = self.build();
        thread::spawn(move || {
            thread::sleep(duration);
            drop(guard);
            std::process::exit(0);
        });
    }
}

impl Default for ThreadsGuardBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[must_use = "guard is dropped immediately without printing statistics"]
pub struct ThreadsGuard {
    start_time: Instant,
    format: Format,
    output_path: Option<PathBuf>,
}

impl ThreadsGuard {
    pub fn new() -> Self {
        init_threads_monitoring();
        #[cfg(feature = "hotpath-alloc-meta")]
        crate::functions::alloc::core::init_thread_alloc_tracking();
        Self {
            start_time: Instant::now(),
            format: Format::default(),
            output_path: None,
        }
    }

    pub fn format(mut self, format: Format) -> Self {
        self.format = format;
        self
    }

    pub fn output_path(mut self, path: impl AsRef<std::path::Path>) -> Self {
        self.output_path = Some(resolve_output_path(path));
        self
    }
}

impl Default for ThreadsGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ThreadsGuard {
    fn drop(&mut self) {
        let elapsed = self.start_time.elapsed();
        let threads_json = get_threads_json();

        let output = crate::output::OutputDestination::from_path(self.output_path.take());
        let mut writer: Box<dyn Write> = match output.writer() {
            Ok(w) => w,
            Err(e) => {
                eprintln!("Failed to create output writer: {}", e);
                return;
            }
        };

        if threads_json.threads.is_empty() {
            let _ = writeln!(writer, "\nNo thread metrics collected.");
            return;
        }

        let format = if std::env::var("HOTPATH_META_OUTPUT_FORMAT").is_ok() {
            Format::from_env()
        } else {
            self.format
        };

        match format {
            Format::None => (),
            Format::Table => {
                let _ = writeln!(
                    writer,
                    "\n=== Thread Statistics (runtime: {:.2}s) ===",
                    elapsed.as_secs_f64()
                );

                let has_alloc = threads_json.threads.iter().any(|t| t.alloc_bytes.is_some());

                let mut header = vec![
                    Cell::new("Thread"),
                    Cell::new("Status"),
                    Cell::new("CPU%"),
                    Cell::new("CPU User"),
                    Cell::new("CPU Sys"),
                    Cell::new("CPU Total"),
                ];
                if has_alloc {
                    header.push(Cell::new("Alloc"));
                    header.push(Cell::new("Dealloc"));
                    header.push(Cell::new("Diff"));
                }

                let mut table = Table::new();
                table.add_row(Row::new(header));

                for thread in &threads_json.threads {
                    let cpu_pct = thread.cpu_percent.as_deref().unwrap_or("-");

                    let mut row = vec![
                        Cell::new(&thread.name),
                        Cell::new(&thread.status),
                        Cell::new(cpu_pct),
                        Cell::new(&thread.cpu_user),
                        Cell::new(&thread.cpu_sys),
                        Cell::new(&thread.cpu_total),
                    ];
                    if has_alloc {
                        row.push(Cell::new(thread.alloc_bytes.as_deref().unwrap_or("-")));
                        row.push(Cell::new(thread.dealloc_bytes.as_deref().unwrap_or("-")));
                        row.push(Cell::new(thread.mem_diff.as_deref().unwrap_or("-")));
                    }

                    table.add_row(Row::new(row));
                }

                let mut summary_parts = Vec::new();
                if let Some(rss) = &threads_json.rss_bytes {
                    summary_parts.push(format!("RSS: {}", rss));
                }
                if let Some(alloc) = &threads_json.total_alloc_bytes {
                    summary_parts.push(format!("Alloc: {}", alloc));
                }
                if let Some(dealloc) = &threads_json.total_dealloc_bytes {
                    summary_parts.push(format!("Dealloc: {}", dealloc));
                }
                if let Some(diff) = &threads_json.alloc_dealloc_diff {
                    summary_parts.push(format!("Diff: {}", diff));
                }

                let summary = if summary_parts.is_empty() {
                    String::new()
                } else {
                    format!(", {}", summary_parts.join(", "))
                };

                let _ = writeln!(
                    writer,
                    "\nThreads ({}{}):",
                    threads_json.thread_count, summary
                );
                let _ = table.print(&mut writer);
            }
            Format::Json => match serde_json::to_string(&threads_json) {
                Ok(json) => {
                    let _ = writeln!(writer, "{}", json);
                }
                Err(e) => eprintln!("Failed to serialize statistics to JSON: {}", e),
            },
            Format::JsonPretty => match serde_json::to_string_pretty(&threads_json) {
                Ok(json) => {
                    let _ = writeln!(writer, "{}", json);
                }
                Err(e) => eprintln!("Failed to serialize statistics to pretty JSON: {}", e),
            },
        }
    }
}
