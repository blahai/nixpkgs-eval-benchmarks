use serde::{
    Deserialize,
    Serialize,
};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MemoryInfoError {
    #[error("Failed to enumerate DMI devices: {0}")]
    EnumerationFailed(String),

    #[error("No memory devices found in DMI data")]
    NoDevicesFound,

    #[error("Failed to parse memory property: {property}")]
    PropertyParseError { property: String },
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct MemoryModule {
    pub memory_type: String,
    pub form_factor: String,
    pub size_bytes: u64,
    pub speed_mts: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct MemoryInfo {
    pub memory_type: String,
    pub speed_mts: u32,
    pub num_modules: usize,
    pub total_size_bytes: u64,
    pub form_factor: String,
    pub modules: Vec<MemoryModule>,
}

/// This is actual malware but still less scuffed than piping udevadm to awk
fn parse_udev(props: &HashMap<String, String>) -> Result<MemoryInfo, MemoryInfoError> {
    let num_devices: usize = props
        .get("MEMORY_ARRAY_NUM_DEVICES")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let mut modules = Vec::new();
    let mut total_size = 0u64;
    let mut memory_type = String::new();
    let mut speed_mts = 0u32;
    let mut form_factor = String::new();

    for i in 0..num_devices {
        let prefix = format!("MEMORY_DEVICE_{i}_");

        let size = props
            .get(&format!("{prefix}SIZE"))
            .and_then(|s| s.parse::<u64>().ok())
            .ok_or_else(|| MemoryInfoError::PropertyParseError {
                property: format!("{prefix}SIZE"),
            })?;

        let mtype = props
            .get(&format!("{prefix}TYPE"))
            .ok_or_else(|| MemoryInfoError::PropertyParseError {
                property: format!("{prefix}TYPE"),
            })?
            .clone();

        let speed = props
            .get(&format!("{prefix}SPEED_MTS"))
            .and_then(|s| s.parse::<u32>().ok())
            .ok_or_else(|| MemoryInfoError::PropertyParseError {
                property: format!("{prefix}SPEED_MTS"),
            })?;

        let form = props
            .get(&format!("{prefix}FORM_FACTOR"))
            .ok_or_else(|| MemoryInfoError::PropertyParseError {
                property: format!("{prefix}FORM_FACTOR"),
            })?
            .clone();

        memory_type.clone_from(&mtype);
        speed_mts = speed;
        form_factor.clone_from(&form);
        total_size += size;

        modules.push(MemoryModule {
            memory_type: mtype,
            form_factor: form,
            size_bytes: size,
            speed_mts: speed,
        });
    }

    if modules.is_empty() {
        return Err(MemoryInfoError::NoDevicesFound);
    }

    Ok(MemoryInfo {
        memory_type,
        speed_mts,
        num_modules: modules.len(),
        total_size_bytes: total_size,
        form_factor,
        modules,
    })
}

pub fn get_mem_stats() -> Result<MemoryInfo, MemoryInfoError> {
    let mut enumerator =
        udev::Enumerator::new().map_err(|e| MemoryInfoError::EnumerationFailed(e.to_string()))?;

    enumerator
        .match_subsystem("dmi")
        .map_err(|e| MemoryInfoError::EnumerationFailed(e.to_string()))?;

    if let Some(device) = enumerator
        .scan_devices()
        .map_err(|e| MemoryInfoError::EnumerationFailed(e.to_string()))?
        .next()
    {
        let mut props = HashMap::new();

        for property in device.properties() {
            let key = property.name().to_string_lossy().to_string();
            let val = property.value().to_string_lossy().to_string();
            props.insert(key, val);
        }

        return parse_udev(&props);
    }

    Err(MemoryInfoError::NoDevicesFound)
}
