use futures::future::join_all;
use indicatif::ProgressStyle;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::process::Command;
use tokio::sync::Semaphore;
use tokio::task;
use tokio::time::{
    Instant,
    sleep,
};
use tracing::{
    debug,
    error,
    info,
    info_span,
    warn,
};
use tracing_indicatif::span_ext::IndicatifSpanExt;

use crate::util::EvalResult;

pub fn sort_nix_paths(paths: Vec<String>) -> Vec<String> {
    let mut sorted = paths;
    sorted.sort_by(|a, b| {
        let name_a = strip_nix_hash(a).unwrap_or("");
        let name_b = strip_nix_hash(b).unwrap_or("");
        name_a.cmp(name_b)
    });
    sorted
}

fn strip_nix_hash(path: &str) -> Option<&str> {
    path.strip_prefix("/nix/store/")?
        .split_once('-')
        .map(|(_, name)| name)
}

pub fn standardize_version(version_str: &str) -> Option<String> {
    let (package, version_part) = version_str.split_once('-')?;

    if !matches!(package, "nix" | "lix") {
        return None;
    }

    let mut major = String::new();
    let mut minor = String::new();
    let mut found_dot = false;

    for c in version_part.chars() {
        if c.is_ascii_digit() {
            if found_dot {
                minor.push(c);
            } else {
                major.push(c);
            }
        } else if c == '.' && !found_dot && !major.is_empty() {
            found_dot = true;
        } else if found_dot && !minor.is_empty() {
            break;
        } else if !found_dot {
            return None;
        }
    }

    if major.is_empty() || minor.is_empty() {
        return None;
    }

    Some(format!("{package}-{major}.{minor}"))
}

pub async fn run_nix_eval(
    nix_path: String,
    nixpkgs: &Path,
    attrpaths: &Path,
    chunk_count: usize,
    cpu_count: usize,
    total_runs: usize,
) -> EvalResult {
    let nix_env = format!("{nix_path}/bin/nix-env");
    let nix_version = strip_nix_hash(&nix_path)
        .and_then(standardize_version)
        .unwrap();
    info!("Running on version {nix_version}");

    let mut results = EvalResult {
        version: nix_version,
        avg_time: Duration::ZERO,
        median_time: Duration::ZERO,
        run_times: vec![],
    };

    for run in 1..=total_runs {
        info!("Run {run}/{total_runs}\n");

        let semaphore = Arc::new(Semaphore::new(cpu_count));
        let start = Instant::now();

        info!("Evaluating {chunk_count} chunks at P{cpu_count}...\n");

        let header_span = info_span!("header");
        header_span.pb_set_style(&ProgressStyle::with_template("{bar} {pos}/{len} {msg}").unwrap());
        header_span.pb_set_length(chunk_count as u64);
        header_span.pb_set_message("Evaluating chunks... (elapsed 0.0s)");
        header_span.pb_set_finish_message("");

        let header_span_enter = header_span.enter();

        let clock_span = header_span.clone();
        let clock_start = start;
        let clock_handle = task::spawn(async move {
            loop {
                sleep(Duration::from_millis(250)).await;
                let elapsed = clock_start.elapsed().as_secs_f64();
                clock_span.pb_set_message(&format!("Evaluating chunks... (elapsed {elapsed:.1}s)"));
            }
        });

        let handles: Vec<_> = (1..=chunk_count)
            .map(|my_chunk| {
                let semaphore = semaphore.clone();
                let nix_env = nix_env.clone();
                let nixpkgs = nixpkgs.to_path_buf();
                let attrpaths = attrpaths.to_path_buf();
                let header_span = header_span.clone();

                task::spawn(async move {
                    let _permit = semaphore.acquire().await.unwrap();
                    info!("Evaluating chunk {my_chunk}");
                    let chunk_timer = Instant::now();
                    let output = Command::new(&nix_env)
                        .arg("-f")
                        .arg(nixpkgs.join("ci/eval/chunk.nix"))
                        .arg("--option")
                        .arg("restrict-eval")
                        .arg("true")
                        .arg("--option")
                        .arg("allow-import-from-derivation")
                        .arg("false")
                        .arg("--query")
                        .arg("--available")
                        .arg("--out-path")
                        .arg("--json")
                        .arg("--meta")
                        .arg("--show-trace")
                        .arg("--arg")
                        .arg("chunkSize")
                        .arg("15000")
                        .arg("--arg")
                        .arg("myChunk")
                        .arg(format!("{}", &my_chunk))
                        .arg("--arg")
                        .arg("attrpathFile")
                        .arg(&attrpaths)
                        .arg("--arg")
                        .arg("systems")
                        .arg("[ \"x86_64-linux\" ]")
                        .arg("--arg")
                        .arg("includeBroken")
                        .arg("false")
                        .arg("--argstr")
                        .arg("extraNixpkgsConfigJson")
                        .arg("{}")
                        .arg("-I")
                        .arg(&nixpkgs)
                        .arg("-I")
                        .arg(&attrpaths)
                        .stdout(std::process::Stdio::null())
                        .output()
                        .await;

                    let chunk_eval_time = chunk_timer.elapsed();
                    header_span.pb_inc(1);
                    match output {
                        Ok(result) => {
                            if result.status.success() {
                                info!(
                                    "Evalation of chunk {my_chunk} finished in {:.3}s",
                                    chunk_eval_time.as_secs_f64()
                                );
                            } else {
                                warn!(
                                "Evaluating chunk {my_chunk} finished in {:.3}s with error: {:#}",
                                chunk_eval_time.as_secs_f64(),
                                String::from_utf8_lossy(&result.stderr)
                            );
                            }
                        }
                        Err(e) => {
                            error!(
                                "Evaluating chunk {my_chunk} failed in {:.3}s with error: {e:#?}",
                                chunk_eval_time.as_secs_f64()
                            );
                        }
                    }
                })
            })
            .collect();

        let _ = join_all(handles).await;

        let elapsed = start.elapsed();

        clock_handle.abort();
        std::mem::drop(header_span_enter);
        std::mem::drop(header_span);

        results.run_times.push(elapsed);
        info!(
            "Total time for run {run}/{total_runs} elapsed: {:.3}s\n",
            elapsed.as_secs_f64()
        );
    }

    results.avg_time = if results.run_times.is_empty() {
        Duration::ZERO
    } else {
        let total: Duration = results.run_times.iter().sum();
        total / results.run_times.len() as u32
    };

    results.median_time = if results.run_times.is_empty() {
        Duration::ZERO
    } else {
        let mut sorted = results.run_times.clone();
        sorted.sort();
        let mid = sorted.len() / 2;
        if sorted.len().is_multiple_of(2) {
            (sorted[mid - 1] + sorted[mid]) / 2
        } else {
            sorted[mid]
        }
    };

    info!("{results:#?}\n");

    results
}

// FIXME: yeet this
pub async fn run_debug(
    nix_path: String,
    chunk_count: usize,
    cpu_count: usize,
    total_runs: usize,
) -> EvalResult {
    let nix_version = strip_nix_hash(&nix_path)
        .and_then(standardize_version)
        .unwrap();
    debug!("Running on version {nix_version}");

    let mut results = EvalResult {
        version: nix_version,
        avg_time: Duration::ZERO,
        median_time: Duration::ZERO,
        run_times: vec![],
    };

    for run in 1..=total_runs {
        println!("Run {run}/{total_runs}\n");
        let semaphore = Arc::new(Semaphore::new(cpu_count));
        let start = Instant::now();
        println!("Evaluating {chunk_count} chunks at P{cpu_count}...\n");

        let handles: Vec<_> = (1..=chunk_count)
            .map(|my_chunk| {
                let semaphore = semaphore.clone();
                task::spawn(async move {
                    let _permit = semaphore.acquire().await.unwrap();
                    println!("Evaulating chunk {my_chunk}");
                    let chunk_timer = Instant::now();
                    sleep(Duration::from_millis(my_chunk as u64 * 20)).await;
                    let chunk_eval_time = chunk_timer.elapsed();
                    println!(
                        "Evalation of chunk {my_chunk} finished in {:.5}ms",
                        chunk_eval_time.as_millis()
                    );
                })
            })
            .collect();

        let _ = join_all(handles).await;
        let elapsed = start.elapsed();
        results.run_times.push(elapsed);
        println!(
            "Total time for run {run}/{total_runs} elapsed: {:.5}ms\n",
            elapsed.as_millis()
        );
    }

    results.avg_time = if results.run_times.is_empty() {
        Duration::ZERO
    } else {
        let total: Duration = results.run_times.iter().sum();
        total / results.run_times.len() as u32
    };

    results.median_time = if results.run_times.is_empty() {
        Duration::ZERO
    } else {
        let mut sorted = results.run_times.clone();
        sorted.sort();
        let mid = sorted.len() / 2;
        if sorted.len().is_multiple_of(2) {
            (sorted[mid - 1] + sorted[mid]) / 2
        } else {
            sorted[mid]
        }
    };

    results
}
