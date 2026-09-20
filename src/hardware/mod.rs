use serde::{Deserialize, Serialize};
use sysinfo::System;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PeerCapabilities {
    pub device_name: String,
    pub is_apple_silicon: bool,
    pub unified_memory: bool,
    pub total_ram_gb: f32,
    pub available_ram_gb: f32,
    pub cpu_cores: usize,
    pub estimated_tflops: f32,
    pub memory_bandwidth_gbps: f32,
    pub supported_quantizations: Vec<String>,
}

impl PeerCapabilities {
    /// Auto-detect system hardware with first-class Apple Silicon Unified Memory awareness
    pub fn detect() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();

        let total_ram_bytes = sys.total_memory();
        let available_ram_bytes = sys.available_memory();
        let total_ram_gb = total_ram_bytes as f32 / (1024.0 * 1024.0 * 1024.0);
        let available_ram_gb = available_ram_bytes as f32 / (1024.0 * 1024.0 * 1024.0);
        let cpu_cores = sys.cpus().len();

        let (device_name, is_apple_silicon, unified_memory, estimated_tflops, memory_bandwidth_gbps) =
            detect_platform_hardware(&sys);

        let supported_quantizations = vec![
            "q4_k_m".to_string(),
            "q8_0".to_string(),
            "fp8".to_string(),
            "fp16".to_string(),
        ];

        Self {
            device_name,
            is_apple_silicon,
            unified_memory,
            total_ram_gb,
            available_ram_gb,
            cpu_cores,
            estimated_tflops,
            memory_bandwidth_gbps,
            supported_quantizations,
        }
    }

    /// Maximum model parameter size (in billions) this node can host in unified memory
    pub fn max_hostable_parameters_4bit(&self) -> f32 {
        // At 4-bit (0.5 bytes/param) + 20% KV-cache headroom, 1B params ~ 0.6 GB
        // We use up to 75% of available unified memory
        let usable_ram = self.available_ram_gb * 0.75;
        usable_ram / 0.65
    }
}

fn detect_platform_hardware(sys: &System) -> (String, bool, bool, f32, f32) {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;

        let brand = Command::new("sysctl")
            .arg("-n")
            .arg("machdep.cpu.brand_string")
            .output()
            .ok()
            .and_then(|out| String::from_utf8(out.stdout).ok())
            .unwrap_or_default()
            .trim()
            .to_string();

        let is_apple_silicon = cfg!(target_arch = "aarch64") || brand.contains("Apple M");

        if is_apple_silicon {
            let (tflops, bandwidth) = estimate_apple_silicon_perf(&brand);
            let name = if !brand.is_empty() {
                brand
            } else {
                "Apple Silicon Mac (Unified Memory)".to_string()
            };
            return (name, true, true, tflops, bandwidth);
        }
    }

    // Fallback for non-macOS or x86
    let cpu_name = sys
        .cpus()
        .first()
        .map(|c| c.brand().to_string())
        .unwrap_or_else(|| "Generic Compute Node".to_string());

    let is_apple = cpu_name.contains("Apple M");
    let tflops = (sys.cpus().len() as f32) * 0.15; // Conservative baseline CPU TFLOPS
    let bandwidth = 50.0; // Standard DDR4/DDR5 baseline

    (cpu_name, is_apple, is_apple, tflops, bandwidth)
}

#[cfg(target_os = "macos")]
fn estimate_apple_silicon_perf(brand: &str) -> (f32, f32) {
    // Return (Estimated FP16 TFLOPS, Unified Memory Bandwidth GB/s)
    if brand.contains("M4 Max") {
        (65.0, 546.0)
    } else if brand.contains("M4 Pro") {
        (45.0, 273.0)
    } else if brand.contains("M4") {
        (38.0, 120.0)
    } else if brand.contains("M3 Max") {
        (58.0, 400.0)
    } else if brand.contains("M3 Pro") {
        (35.0, 150.0)
    } else if brand.contains("M3") {
        (28.0, 100.0)
    } else if brand.contains("M2 Ultra") {
        (55.0, 800.0)
    } else if brand.contains("M2 Max") {
        (28.0, 400.0)
    } else if brand.contains("M2 Pro") {
        (16.0, 200.0)
    } else if brand.contains("M2") {
        (12.0, 100.0)
    } else if brand.contains("M1 Ultra") {
        (42.0, 800.0)
    } else if brand.contains("M1 Max") {
        (21.0, 400.0)
    } else if brand.contains("M1 Pro") {
        (10.5, 200.0)
    } else if brand.contains("M1") {
        (5.2, 68.25)
    } else {
        (15.0, 100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capabilities_detection() {
        let caps = PeerCapabilities::detect();
        assert!(caps.total_ram_gb > 0.0);
        assert!(caps.cpu_cores > 0);
        assert!(caps.estimated_tflops > 0.0);
        println!(
            "Detected Hardware: {} (UMA: {}, RAM: {:.1} GB, TFLOPS: {:.1}, Max 4-bit Params: {:.1}B)",
            caps.device_name,
            caps.unified_memory,
            caps.total_ram_gb,
            caps.estimated_tflops,
            caps.max_hostable_parameters_4bit()
        );
    }
}
