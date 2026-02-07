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
    breadcrumb_label: &'static str,
}

const SEO_MAPPINGS: &[SeoConfig] = &[
    SeoConfig {
        path: "/",
        title: "hotpath-rs | Rust Async Profiler for Memory & Performance",
        description: "hotpath-rs is a lightweight Rust profiler for performance, memory, and async data flow. Profile functions, channels, futures, and streams with zero overhead.",
        breadcrumb_label: "Home",
    },
    SeoConfig {
        path: "/sampling_comparison",
        title: "Rust Profiler Comparison: Instrumentation vs Sampling | hotpath-rs",
        description: "Compare hotpath-rs with sampling profilers like perf, flamegraph, and samply. See how results differ for CPU-bound, blocking I/O, and async Rust workloads.",
        breadcrumb_label: "Sampling Comparison",
    },
    SeoConfig {
        path: "/profiling_modes",
        title: "Rust Profiling Modes: Static Reports & Live TUI | hotpath-rs",
        description: "Choose between static performance reports and a live TUI dashboard for real-time Rust profiling. Analyze timing, memory, and data flow metrics with hotpath-rs.",
        breadcrumb_label: "Profiling Modes",
    },
    SeoConfig {
        path: "/functions",
        title: "Rust Function Profiler: Timing & Memory Measurement | hotpath-rs",
        description: "Profile Rust functions with precise timing, memory allocation tracking, and percentile statistics. Measure sync and async functions with the #[measure] macro.",
        breadcrumb_label: "Functions",
    },
    SeoConfig {
        path: "/futures",
        title: "Async Rust Profiler: Futures Monitoring & Poll Tracking | hotpath-rs",
        description: "Monitor async Rust futures with poll counts, completion tracking, and value logging. Debug async bottlenecks and understand future execution patterns.",
        breadcrumb_label: "Futures",
    },
    SeoConfig {
        path: "/channels",
        title: "Rust Channel Profiler: Track Message Flow & Throughput | hotpath-rs",
        description: "Track Rust channel message flow with hotpath-rs. Monitor tokio, crossbeam, futures, and std channels with send/receive counts, queue sizes, and throughput.",
        breadcrumb_label: "Channels",
    },
    SeoConfig {
        path: "/streams",
        title: "Rust Stream Profiler: Async Stream Monitoring | hotpath-rs",
        description: "Profile async Rust streams with item counts, throughput metrics, and optional item logging. Instrument futures::Stream with the hotpath::stream! macro.",
        breadcrumb_label: "Streams",
    },
    SeoConfig {
        path: "/threads",
        title: "Rust Thread Profiler: CPU & Memory per Thread | hotpath-rs",
        description: "Monitor per-thread CPU and memory usage in Rust applications. Track thread states, allocation counts, and system time with the hotpath-rs TUI dashboard.",
        breadcrumb_label: "Threads",
    },
    SeoConfig {
        path: "/github_ci",
        title: "Rust Performance CI: Automated PR Benchmarking | hotpath-rs",
        description: "Automate Rust performance benchmarking in GitHub Actions. Detect performance regressions on every pull request with hotpath-rs CI integration.",
        breadcrumb_label: "GitHub CI",
    },
    SeoConfig {
        path: "/mcp",
        title: "MCP Integration: Query Rust Profiling Data with LLMs | hotpath-rs",
        description: "Connect LLM agents like Claude Code to live Rust profiling data via MCP. Query performance, memory usage, and async operations in natural language.",
        breadcrumb_label: "MCP Integration",
    },
];

const STATIC_EXTENSIONS: &[&str] = &[
    ".css", ".js", ".png", ".jpg", ".jpeg", ".gif", ".svg", ".ico", ".woff", ".woff2", ".ttf",
    ".eot", ".mp4", ".webm",
];

const BASE_URL: &str = "https://hotpath.rs";
const OG_IMAGE: &str = "https://hotpath.rs/images/hotpath-ferris.png";
const SOFTWARE_APP_JSON_LD: &str = r#"{"@context":"https://schema.org","@type":"SoftwareApplication","name":"hotpath-rs","applicationCategory":"DeveloperApplication","operatingSystem":"Linux, macOS, Windows","programmingLanguage":"Rust","description":"A real-time Rust profiler for performance, memory allocations, async futures, channels, and streams.","url":"https://hotpath.rs","downloadUrl":"https://crates.io/crates/hotpath","codeRepository":"https://github.com/pawurb/hotpath-rs","license":"https://opensource.org/licenses/MIT","author":{"@type":"Person","name":"Pawel Urbanek","url":"https://pawelurbanek.com"},"offers":{"@type":"Offer","price":"0","priceCurrency":"USD"}}"#;

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
            r#"<meta name="description" content="{}">
    <link rel="canonical" href="{}">
    <meta property="og:title" content="{}">
    <meta property="og:description" content="{}">
    <meta property="og:url" content="{}">
    <meta property="og:type" content="website">
    <meta property="og:image" content="{}">
    <meta property="og:site_name" content="hotpath-rs">
    <meta property="og:locale" content="en_US">
    <meta name="twitter:card" content="summary_large_image">
    <meta name="twitter:title" content="{}">
    <meta name="twitter:description" content="{}">
    <meta name="twitter:image" content="{}">"#,
            config.description,
            canonical_url,
            config.title,
            config.description,
            canonical_url,
            OG_IMAGE,
            config.title,
            config.description,
            OG_IMAGE,
        ),
    );

    let h1_regex = Regex::new(r#"<h1 class="menu-title">([^<]*)</h1>"#).unwrap();
    let modified = h1_regex.replace(&modified, r#"<div class="menu-title">$1</div>"#);

    let mut json_ld_block = String::new();

    let breadcrumb =
        build_breadcrumb_json_ld(config.breadcrumb_label, &canonical_url, config.path == "/");
    json_ld_block.push_str(&format!(
        r#"<script type="application/ld+json">{}</script>"#,
        breadcrumb
    ));

    let entity_json_ld = if config.path == "/" {
        SOFTWARE_APP_JSON_LD.to_string()
    } else {
        build_tech_article_json_ld(config, &canonical_url)
    };
    json_ld_block.push_str(&format!(
        r#"<script type="application/ld+json">{}</script>"#,
        entity_json_ld
    ));

    let modified = modified.replace("</head>", &format!("{}</head>", json_ld_block));

    Response::from_parts(parts, Body::from(modified)).into_response()
}

fn build_breadcrumb_json_ld(label: &str, canonical_url: &str, is_home: bool) -> String {
    if is_home {
        return format!(
            r#"{{"@context":"https://schema.org","@type":"BreadcrumbList","itemListElement":[{{"@type":"ListItem","position":1,"name":"Home","item":"{}"}}]}}"#,
            canonical_url
        );
    }

    format!(
        r#"{{"@context":"https://schema.org","@type":"BreadcrumbList","itemListElement":[{{"@type":"ListItem","position":1,"name":"Home","item":"https://hotpath.rs/"}},{{"@type":"ListItem","position":2,"name":"{}","item":"{}"}}]}}"#,
        label, canonical_url
    )
}

fn build_tech_article_json_ld(config: &SeoConfig, canonical_url: &str) -> String {
    let headline = config.title.split(" | ").next().unwrap_or(config.title);
    format!(
        r#"{{"@context":"https://schema.org","@type":"TechArticle","headline":"{}","description":"{}","image":"{}","datePublished":"2025-01-15","dateModified":"2026-02-07","author":{{"@type":"Person","name":"Pawel Urbanek"}},"publisher":{{"@type":"Organization","name":"hotpath-rs"}},"mainEntityOfPage":"{}"}}"#,
        headline, config.description, OG_IMAGE, canonical_url
    )
}
