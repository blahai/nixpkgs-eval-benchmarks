use serde::{
    Deserialize,
    Serialize,
};
use tokio::time::Duration;

pub mod nix;
pub mod specs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EvalResult {
    pub version: String,
    pub avg_time: Duration,
    pub median_time: Duration,
    pub run_times: Vec<Duration>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct HostSpecs {
    pub cpu: specs::cpu::CpuInfo,
    pub mem: specs::memory::MemoryInfo,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MiscInfo {
    pub nixpkgs_rev: String,
    pub debug: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FinalReport {
    pub host_specs: HostSpecs,
    pub misc: MiscInfo,
    pub runs: Vec<EvalResult>,
}
