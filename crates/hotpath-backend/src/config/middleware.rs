use axum::{
    body::Body,
    extract::Request,
    http::HeaderValue,
    middleware::Next,
    response::{IntoResponse, Response},
};
use http_body_util::BodyExt;
use regex::Regex;
use std::time::Instant;
use tower_http::cors::{Any, CorsLayer};
use tracing::Instrument;
use tracing::info_span;
use uuid::Uuid;

struct SeoConfig {
    path: &'static str,
    title: &'static str,
    description: &'static str,
}

const SEO_MAPPINGS: &[SeoConfig] = &[
    SeoConfig {
        path: "/",
        title: "hotpath-rs - A Simple Async Rust Profiler for Performance & Memory",
        description: "hotpath-rs is a real-time performance profiler for Rust applications. Monitor CPU, memory, async operations, and async data flow with minimal configuration.",
    },
    SeoConfig {
        path: "/sampling_comparison",
        title: "hotpath-rs and Sampling Profilers - CPU, Blocking & Async I/O Comparison",
        description: "Compare hotpath-rs instrumentation with traditional sampling profilers. Understand trade-offs for CPU, blocking, and async I/O profiling scenarios.",
    },
    SeoConfig {
        path: "/profiling_modes",
        title: "Profiling Modes - Static Reports & Live TUI Dashboard | hotpath-rs",
        description: "Choose between static HTML reports and live TUI dashboard modes. Analyze performance data offline or watch metrics in real-time.",
    },
    SeoConfig {
        path: "/functions",
        title: "Functions Profiling - Timing & Memory Measurement | hotpath-rs",
        description: "Profile individual Rust functions with precise timing and memory measurements. Track execution counts, durations, and allocations per function.",
    },
    SeoConfig {
        path: "/futures",
        title: "Futures Monitoring - Async Rust Profiling | hotpath-rs",
        description: "Monitor async Rust futures with poll counts and timing data. Debug async bottlenecks and understand future execution patterns.",
    },
    SeoConfig {
        path: "/channels",
        title: "Channels Monitoring - Track Message Flow & Throughput | hotpath-rs",
        description: "Track message flow through Rust channels with throughput metrics. Monitor send/receive operations and identify channel bottlenecks.",
    },
    SeoConfig {
        path: "/streams",
        title: "Streams Monitoring - Async Stream Profiling | hotpath-rs",
        description: "Profile async streams with item counts and timing data. Analyze stream throughput and identify slow stream processing.",
    },
    SeoConfig {
        path: "/threads",
        title: "Threads Monitoring - CPU & Memory per Thread | hotpath-rs",
        description: "Monitor CPU and memory usage per thread in your Rust application. Identify thread-level bottlenecks and resource consumption.",
    },
    SeoConfig {
        path: "/github_ci",
        title: "GitHub CI Integration - Automated PR Benchmarking | hotpath-rs",
        description: "Integrate hotpath-rs into GitHub CI for automated performance benchmarking. Get performance regression alerts on every pull request.",
    },
    SeoConfig {
        path: "/mcp",
        title: "MCP Integration - Query Profiling Data with LLM Agents | hotpath-rs",
        description: "Connect LLM agents like Claude Code to your Rust application's profiling data via MCP. Ask natural language questions about performance, memory, and async operations.",
    },
];

const STATIC_EXTENSIONS: &[&str] = &[
    ".css", ".js", ".png", ".jpg", ".jpeg", ".gif", ".svg", ".ico", ".woff", ".woff2", ".ttf",
    ".eot", ".mp4", ".webm",
];

const BASE_URL: &str = "https://hotpath.rs";

pub async fn request_tracing(request: Request, next: Next) -> Response {
    let path = request.uri().path().to_string();

    // Skip instrumenting static asset requests
    if STATIC_EXTENSIONS.iter().any(|ext| path.ends_with(ext)) {
        return next.run(request).await;
    }

    let uuid = Uuid::new_v4();
    let request_id = &uuid.to_string()[0..12];
    let method = request.method().clone();

    let info_span = info_span!("req", id = %request_id, method = %method, path = %path);

    async move {
        let start = Instant::now();
        let response = next.run(request).await;
        let duration = start.elapsed();

        tracing::info!(
            status = %response.status(),
            duration_ms = duration.as_millis(),
        );

        response
    }
    .instrument(info_span)
    .await
}

pub async fn security_headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;

    let headers = response.headers_mut();
    headers.insert(
        "X-Content-Type-Options",
        HeaderValue::from_static("nosniff"),
    );
    headers.insert("X-Frame-Options", HeaderValue::from_static("SAMEORIGIN"));
    headers.insert(
        "Referrer-Policy",
        HeaderValue::from_static("no-referrer-when-downgrade"),
    );
    headers.insert(
        "Strict-Transport-Security",
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );

    response
}

pub fn cors() -> CorsLayer {
    CorsLayer::new()
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
        ])
        .allow_origin(Any)
        .allow_headers(Any)
}

pub fn init_logs() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "hotpath_backend=debug,tower_http=debug".into());

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_ids(false)
        .with_line_number(true)
        .init();
}

pub async fn seo_titles(request: Request, next: Next) -> Response {
    let path = request.uri().path().to_string();

    let seo_config = SEO_MAPPINGS.iter().find(|cfg| cfg.path == path);

    let response = next.run(request).await;

    let Some(config) = seo_config else {
        return response;
    };

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if !content_type.contains("text/html") {
        return response;
    }

    let (parts, body) = response.into_parts();

    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(_) => return Response::from_parts(parts, Body::empty()),
    };

    let html = String::from_utf8_lossy(&bytes);

    let title_regex = Regex::new(r"<title>[^<]*</title>").unwrap();
    let modified = title_regex.replace(&html, format!("<title>{}</title>", config.title));

    let desc_regex = Regex::new(r#"<meta name="description" content="[^"]*">"#).unwrap();
    let canonical_url = format!("{}{}", BASE_URL, path);
    let modified = desc_regex.replace(
        &modified,
        format!(
            "<meta name=\"description\" content=\"{}\">\n    <link rel=\"canonical\" href=\"{}\">",
            config.description, canonical_url
        ),
    );

    Response::from_parts(parts, Body::from(modified.into_owned())).into_response()
}
