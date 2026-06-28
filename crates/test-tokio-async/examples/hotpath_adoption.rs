//! Finds public GitHub repos that depend on the `hotpath` crate.
//!
//! Two phases:
//! 1. Code search for `hotpath` in `Cargo.toml` files (catches anything that
//!    merely mentions the word, including false positives).
//! 2. Fetch each candidate `Cargo.toml`, parse it as TOML, and keep only repos
//!    that declare `hotpath` in a real dependency table.
//!
//! Results are merged into a JSON file (default `hotpath_adoption.json` in the
//! repo root, override with
//! `HOTPATH_ADOPTION_PATH`). The first run stamps every repo with `discovered_at`;
//! later daily runs keep each repo's original `discovered_at`, refresh its `stars`
//! / `requirement`, and bump `last_seen_at`. Repos that drop out of the search are
//! retained untouched (their stale `last_seen_at` flags them).
//!
//! Run with:
//! `GH_TOKEN="$(gh auth token)" cargo run -p test-tokio-async --release --example hotpath_adoption`

use anyhow::{Context, Result};
use chrono::{SecondsFormat, Utc};
use futures::stream::{self, StreamExt};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::Duration;

/// The hotpath repo itself, skipped (would always self-match).
const SKIP_REPO: &str = "pawurb/hotpath-rs";
/// Concurrent raw-file fetches in phase 2.
const FETCH_CONCURRENCY: usize = 8;
/// Default path for the merged adoption dataset (repo root).
const DEFAULT_OUTPUT_PATH: &str = "hotpath_adoption.json";

#[derive(Debug, Deserialize)]
struct SearchResponse {
    items: Vec<CodeItem>,
}

#[derive(Debug, Deserialize)]
struct CodeItem {
    path: String,
    repository: Repository,
}

#[derive(Debug, Deserialize, Clone)]
struct Repository {
    full_name: String,
    html_url: String,
    fork: bool,
}

#[derive(Debug, Clone)]
struct Candidate {
    full_name: String,
    html_url: String,
    path: String,
}

#[derive(Debug)]
struct Dependent {
    full_name: String,
    html_url: String,
    requirement: String,
    stars: u64,
}

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    full_name: String,
    html_url: String,
    requirement: String,
    stars: u64,
    discovered_at: String,
    last_seen_at: String,
}

#[derive(Debug, Deserialize)]
struct RepoInfo {
    stargazers_count: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let token = std::env::var("GH_TOKEN")
        .or_else(|_| std::env::var("GITHUB_TOKEN"))
        .context("GH_TOKEN (or GITHUB_TOKEN) env var is required")?;

    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static("hotpath-adoption-checker"),
    );
    headers.insert(
        ACCEPT,
        HeaderValue::from_static("application/vnd.github+json"),
    );
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {token}"))?,
    );

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    let candidates = search_candidates(&client).await?;
    println!(
        "Phase 1: {} candidate Cargo.toml files mentioning \"hotpath\".",
        candidates.len()
    );

    let dependents = confirm_dependents(&client, candidates).await;
    println!(
        "Phase 2: {} repos depend on the hotpath crate.",
        dependents.len()
    );

    let path = output_path();
    let mut records = load_records(&path)?;
    let now = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    let (added, updated) = merge_dependents(&mut records, dependents, &now);

    let mut sorted: Vec<&Record> = records.values().collect();
    sorted.sort_by(|a, b| b.stars.cmp(&a.stars).then(a.full_name.cmp(&b.full_name)));
    write_records(&path, &sorted)?;

    println!();
    println!(
        "Merged into {}: {added} new, {updated} refreshed, {} total tracked.",
        path.display(),
        records.len()
    );

    Ok(())
}

fn output_path() -> PathBuf {
    std::env::var("HOTPATH_ADOPTION_PATH")
        .unwrap_or_else(|_| DEFAULT_OUTPUT_PATH.to_string())
        .into()
}

fn load_records(path: &std::path::Path) -> Result<HashMap<String, Record>> {
    let body = match std::fs::read_to_string(path) {
        Ok(body) => body,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(HashMap::new()),
        Err(err) => return Err(err).context(format!("reading {}", path.display())),
    };
    let list: Vec<Record> =
        serde_json::from_str(&body).context(format!("parsing {}", path.display()))?;
    Ok(list.into_iter().map(|r| (r.full_name.clone(), r)).collect())
}

fn write_records(path: &std::path::Path, records: &[&Record]) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let json = serde_json::to_string_pretty(records)?;
    std::fs::write(path, json).context(format!("writing {}", path.display()))?;
    Ok(())
}

/// Merge this run's dependents into the persisted map, preserving each repo's
/// original `discovered_at`. Returns `(new, refreshed)` counts.
fn merge_dependents(
    records: &mut HashMap<String, Record>,
    dependents: Vec<Dependent>,
    now: &str,
) -> (usize, usize) {
    let (mut added, mut updated) = (0, 0);
    for dep in dependents {
        match records.get_mut(&dep.full_name) {
            Some(existing) => {
                existing.html_url = dep.html_url;
                existing.requirement = dep.requirement;
                existing.stars = dep.stars;
                existing.last_seen_at = now.to_string();
                updated += 1;
            }
            None => {
                records.insert(
                    dep.full_name.clone(),
                    Record {
                        full_name: dep.full_name,
                        html_url: dep.html_url,
                        requirement: dep.requirement,
                        stars: dep.stars,
                        discovered_at: now.to_string(),
                        last_seen_at: now.to_string(),
                    },
                );
                added += 1;
            }
        }
    }
    (added, updated)
}

async fn search_candidates(client: &reqwest::Client) -> Result<Vec<Candidate>> {
    let query = r#"hotpath filename:Cargo.toml"#;
    let encoded_query = urlencoding::encode(query);

    let mut seen: HashSet<(String, String)> = HashSet::new();
    let mut candidates = Vec::new();

    // Code search caps at 1000 results (10 pages * 100). The secondary rate
    // limit on code search is strict, so sleep between pages.
    for page in 1..=10 {
        let url = format!(
            "https://api.github.com/search/code?q={encoded_query}&per_page=100&page={page}"
        );

        let resp = client.get(&url).send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            eprintln!("search page {page}: HTTP {status}: {body}");
            break;
        }

        let response = resp.json::<SearchResponse>().await?;
        if response.items.is_empty() {
            break;
        }

        for item in response.items {
            if item.repository.fork || item.repository.full_name == SKIP_REPO {
                continue;
            }
            let key = (item.repository.full_name.clone(), item.path.clone());
            if seen.insert(key) {
                candidates.push(Candidate {
                    full_name: item.repository.full_name,
                    html_url: item.repository.html_url,
                    path: item.path,
                });
            }
        }

        tokio::time::sleep(Duration::from_secs(3)).await;
    }

    Ok(candidates)
}

async fn confirm_dependents(
    client: &reqwest::Client,
    candidates: Vec<Candidate>,
) -> Vec<Dependent> {
    let total_files = candidates.len();
    let unique_repo_count = candidates
        .iter()
        .map(|c| c.full_name.as_str())
        .collect::<HashSet<_>>()
        .len();

    let confirmed: Vec<(Candidate, String)> = stream::iter(candidates)
        .map(|cand| async move {
            match fetch_requirement(client, &cand).await {
                Ok(req) => req.map(|r| (cand, r)),
                Err(err) => {
                    eprintln!("fetch {}/{}: {err}", cand.full_name, cand.path);
                    None
                }
            }
        })
        .buffer_unordered(FETCH_CONCURRENCY)
        .filter_map(|res| async move { res })
        .collect()
        .await;

    let mut seen = HashSet::new();
    let unique: Vec<(Candidate, String)> = confirmed
        .into_iter()
        .filter(|(cand, _)| seen.insert(cand.full_name.clone()))
        .collect();

    eprintln!(
        "Breakdown: {total_files} candidate files across {unique_repo_count} unique \
         repos -> {} confirmed as hotpath dependents (the rest mention hotpath but \
         not as a dependency, or are duplicate Cargo.toml files in one repo).",
        unique.len(),
    );

    stream::iter(unique)
        .map(|(cand, requirement)| async move {
            let stars = fetch_stars(client, &cand.full_name).await.unwrap_or(0);
            Dependent {
                full_name: cand.full_name,
                html_url: cand.html_url,
                requirement,
                stars,
            }
        })
        .buffer_unordered(FETCH_CONCURRENCY)
        .collect()
        .await
}

async fn fetch_stars(client: &reqwest::Client, full_name: &str) -> Result<u64> {
    let url = format!("https://api.github.com/repos/{full_name}");
    let info = client
        .get(&url)
        .send()
        .await?
        .error_for_status()?
        .json::<RepoInfo>()
        .await?;
    Ok(info.stargazers_count)
}

async fn fetch_requirement(client: &reqwest::Client, cand: &Candidate) -> Result<Option<String>> {
    let raw_url = format!(
        "https://raw.githubusercontent.com/{}/HEAD/{}",
        cand.full_name, cand.path
    );
    let body = client
        .get(&raw_url)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    let value: toml::Value = match toml::from_str(&body) {
        Ok(v) => v,
        Err(_) => return Ok(None),
    };
    Ok(find_hotpath_dep(&value, false))
}

/// Recursively scan a parsed `Cargo.toml` for a `hotpath` key inside any table
/// whose name ends in `dependencies` (`[dependencies]`, `[dev-dependencies]`,
/// `[workspace.dependencies]`, `[target.'cfg(..)'.dependencies]`, ...).
fn find_hotpath_dep(value: &toml::Value, parent_is_deps: bool) -> Option<String> {
    let table = value.as_table()?;
    for (key, child) in table {
        if parent_is_deps && key == "hotpath" {
            return Some(render_requirement(child));
        }
        let child_is_deps = key.ends_with("dependencies");
        if let Some(found) = find_hotpath_dep(child, child_is_deps) {
            return Some(found);
        }
    }
    None
}

fn render_requirement(value: &toml::Value) -> String {
    match value {
        toml::Value::String(version) => format!("\"{version}\""),
        toml::Value::Table(t) => {
            if let Some(v) = t.get("version").and_then(|v| v.as_str()) {
                format!("\"{v}\"")
            } else if t.get("git").is_some() {
                "git".to_string()
            } else if t.get("path").is_some() {
                "path".to_string()
            } else if t.get("workspace").and_then(|v| v.as_bool()) == Some(true) {
                "workspace".to_string()
            } else {
                "{ .. }".to_string()
            }
        }
        other => other.to_string(),
    }
}
