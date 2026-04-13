use crate::util::nix::{
    run_debug,
    run_nix_eval,
    sort_nix_paths,
};
use crate::util::specs::cpu::get_cpu_stats;
use crate::util::specs::memory::get_mem_stats;
use crate::util::{
    FinalReport,
    HostSpecs,
    MiscInfo,
};
use chrono::Utc;
use clap::Parser;
use inquire::MultiSelect;
use serde_json::{
    self,
    Value,
};
use std::path::PathBuf;
use std::thread;
use tokio::fs::{
    File,
    read_to_string,
};
use tokio::io::AsyncWriteExt;
use tracing::{
    debug,
    error,
    info,
    warn,
};
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod util;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(long)]
    nix_paths: PathBuf,
    #[arg(long)]
    nixpkgs: PathBuf,
    #[arg(long)]
    attrpaths: PathBuf,
    #[arg(long)]
    chunk_size: usize,
    #[arg(long, short, default_value_t = 1)]
    runs: usize,
    #[arg(long, default_value_t = false)]
    debug: bool,
}

fn value_len(v: &Value) -> usize {
    match v {
        Value::Array(arr) => arr.len(),
        Value::Object(obj) => obj.len(),
        Value::String(s) => s.len(),
        _ => 0,
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let indicatif_layer = IndicatifLayer::new();
    let filter = tracing_subscriber::EnvFilter::new("off,nixbench=debug");
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().with_writer(indicatif_layer.get_stderr_writer()))
        .with(indicatif_layer)
        .init();

    let cli = Cli::parse();

    let mut nix_paths_json: Vec<String> =
        serde_json::from_str(&read_to_string(cli.nix_paths).await?)?;
    nix_paths_json.sort();
    nix_paths_json.dedup();

    let nix_paths = sort_nix_paths(nix_paths_json);

    debug!("Found nix paths: {nix_paths:#?}");
    let nixpkgs = cli.nixpkgs;
    debug!("Found nixpkgs: {nixpkgs:#?}");
    let attrpaths = cli.attrpaths.clone();
    debug!("Found attrpaths: {attrpaths:#?}");
    let attrpaths_json: Value = serde_json::from_str(&read_to_string(cli.attrpaths).await?)?;
    let attrpaths_len = value_len(&attrpaths_json);
    debug!("attrpaths length: {attrpaths_len}");
    let runs = cli.runs;
    debug!("Running each eval {runs} times");
    if runs < 2 {
        warn!(
            "WARNING: you have set the run count to less than 2, this is probably fine but can cause issues with outlier data"
        );
    }

    let chunk_count = (attrpaths_len - 1) / cli.chunk_size + 1;
    debug!("Chunk count: {chunk_count}");

    let cpu_count = thread::available_parallelism()?.get();

    let host_specs = HostSpecs {
        cpu: get_cpu_stats().unwrap_or_default(),
        mem: get_mem_stats().unwrap_or_default(),
    };

    let ans = MultiSelect::new("Select all the nix versions to eval:", nix_paths.clone())
        .with_all_selected_by_default()
        .with_page_size(nix_paths.len())
        .with_help_message("Space to (de)select, enter to confirm")
        .prompt();

    let filtered_nix_paths = match ans {
        Ok(r) => r,
        Err(e) => {
            error!("something exploded while selecting nix paths, falling back to all {e:#?}");
            nix_paths.clone()
        }
    };

    let misc = MiscInfo {
        nixpkgs_rev: String::from("TODO"),
        debug: cli.debug,
    };

    let mut results = FinalReport {
        host_specs,
        misc,
        runs: Vec::new(),
    };

    let mut run_counter = 0;

    let nonce = Utc::now().timestamp();
    for nix_path in filtered_nix_paths.clone() {
        let run = if cli.debug {
            run_debug(nix_path, chunk_count, cpu_count, runs).await
        } else {
            run_nix_eval(nix_path, &nixpkgs, &attrpaths, chunk_count, cpu_count, runs).await
        };

        run_counter += 1;
        info!(
            "Eval {run_counter}/{} done",
            (filtered_nix_paths.len() * runs)
        );

        results.runs.push(run);
    }

    info!("{results:#?}");
    let filename = format!("data/{nonce}.json");
    let json = serde_json::to_string_pretty(&results).unwrap();
    let mut file = File::create(&filename).await?;
    file.write_all(json.as_bytes()).await?;

    info!("Data written to {filename}");

    Ok(())
}
