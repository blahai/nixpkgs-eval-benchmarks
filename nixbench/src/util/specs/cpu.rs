use serde::{
    Deserialize,
    Serialize,
};
use std::collections::HashMap;
use std::fs;
use std::string::ToString;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct CpuInfo {
    pub vendor_id: String,
    pub physical_cpus: usize,
    pub cpu_cores: usize,
    pub max_freq_mhz: f64,
    pub l3_cache_kb: u64,
}

pub fn get_cpu_stats() -> Result<CpuInfo, Box<dyn std::error::Error>> {
    let mut cpu_info = CpuInfo::default();

    let cpuinfo_content = fs::read_to_string("/proc/cpuinfo")?;
    let mut cpuinfo_map = HashMap::new();

    for line in cpuinfo_content.lines() {
        if line.is_empty() {
            break;
        }
        if let Some((key, value)) = line.split_once(':') {
            cpuinfo_map.insert(key.trim().to_string(), value.trim().to_string());
        }
    }

    cpu_info.vendor_id = cpuinfo_map.get("vendor_id").cloned().unwrap_or_default();
    cpu_info.max_freq_mhz = fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq")?
            .trim()
            .parse::<f64>()?
            // returns KHz, I want MHz
            / 1_000.0;

    cpu_info.l3_cache_kb =
        fs::read_to_string("/sys/devices/system/cpu/cpu0/cache/index3/number_of_sets")?
            .trim()
            .parse::<u64>()?;

    cpu_info.physical_cpus = count_physical_cpus()?;
    cpu_info.cpu_cores = cpuinfo_map
        .get("cpu cores")
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);

    Ok(cpu_info)
}

fn count_physical_cpus() -> std::io::Result<usize> {
    let online = fs::read_to_string("/sys/devices/system/cpu/online")?;
    let count = if online.contains('-') {
        let parts: Vec<&str> = online.trim().split('-').collect();
        if parts.len() == 2 {
            let end: usize = parts[1].parse().unwrap_or(0);
            end + 1
        } else {
            1
        }
    } else {
        online.trim().split(',').count()
    };

    Ok(count)
}
