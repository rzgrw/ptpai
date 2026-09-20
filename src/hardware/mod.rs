use serde::{Deserialize, Serialize};
use sysinfo::System;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PeerCapabilities {
    pub device_name: String,
    pub gpu_type: String,
    pub backend: String, // "metal", "cuda", "rocm", "webgpu", "cpu"
    pub is_apple_silicon: bool,
    pub is_discrete_gpu: bool,
    pub unified_memory: bool,
    pub total_ram_gb: f32,
    pub available_ram_gb: f32,
    pub vram_gb: f32,
    pub cpu_cores: usize,
    pub estimated_tflops: f32,
    pub memory_bandwidth_gbps: f32,
    pub supported_quantizations: Vec<String>,
}

impl PeerCapabilities {
    /// Auto-detect hardware across Apple Silicon macOS, Linux (NVIDIA CUDA / AMD ROCm), and x86
    pub fn detect() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();

        let total_ram_bytes = sys.total_memory();
        let available_ram_bytes = sys.available_memory();
        let total_ram_gb = total_ram_bytes as f32 / (1024.0 * 1024.0 * 1024.0);
        let available_ram_gb = available_ram_bytes as f32 / (1024.0 * 1024.0 * 1024.0);
        let cpu_cores = sys.cpus().len();

        let (device_name, gpu_type, backend, is_apple_silicon, is_discrete_gpu, unified_memory, vram_gb, estimated_tflops, memory_bandwidth_gbps) =
            detect_platform_hardware(&sys, total_ram_gb);

        let supported_quantizations = vec![
            "q4_k_m".to_string(),
            "q8_0".to_string(),
            "fp8".to_string(),
            "fp16".to_string(),
        ];

        Self {
            device_name,
            gpu_type,
            backend,
            is_apple_silicon,
            is_discrete_gpu,
            unified_memory,
            total_ram_gb,
            available_ram_gb,
            vram_gb,
            cpu_cores,
            estimated_tflops,
            memory_bandwidth_gbps,
            supported_quantizations,
        }
    }

    /// Maximum model parameter size (in billions) this node can host
    pub fn max_hostable_parameters_4bit(&self) -> f32 {
        let memory = if self.is_discrete_gpu && self.vram_gb > 0.0 {
            self.vram_gb * 0.85
        } else {
            self.available_ram_gb * 0.75
        };
        memory / 0.65
    }
}

fn detect_platform_hardware(sys: &System, total_ram_gb: f32) -> (String, String, String, bool, bool, bool, f32, f32, f32) {
    // 1. macOS Apple Silicon Detection
    #[cfg(target_os = "macos")]
    {
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
            return (
                name,
                "Apple Silicon GPU (Metal)".to_string(),
                "metal".to_string(),
                true,
                false,
                true,
                total_ram_gb, // VRAM == Total Unified RAM on Apple Silicon
                tflops,
                bandwidth,
            );
        }
    }

    // 2. Linux / Windows NVIDIA CUDA GPU Detection via nvidia-smi
    if let Some((gpu_name, vram_mb, tflops, bandwidth)) = detect_nvidia_gpu() {
        let vram_gb = vram_mb / 1024.0;
        return (
            format!("Linux / CUDA Server ({})", gpu_name),
            gpu_name,
            "cuda".to_string(),
            false,
            true,  // Discrete GPU
            false, // Discrete VRAM
            vram_gb,
            tflops,
            bandwidth,
        );
    }

    // 3. Linux AMD ROCm GPU Detection via rocm-smi
    if let Some((gpu_name, vram_mb, tflops, bandwidth)) = detect_amd_gpu() {
        let vram_gb = vram_mb / 1024.0;
        return (
            format!("Linux / ROCm Server ({})", gpu_name),
            gpu_name,
            "rocm".to_string(),
            false,
            true,
            false,
            vram_gb,
            tflops,
            bandwidth,
        );
    }

    // 4. Standard CPU fallback (Intel / AMD x86_64 or generic ARM)
    let cpu_name = sys
        .cpus()
        .first()
        .map(|c| c.brand().to_string())
        .unwrap_or_else(|| "Generic Compute Node".to_string());

    let is_apple = cpu_name.contains("Apple M");
    let tflops = (sys.cpus().len() as f32) * 0.25; // AVX2 / NEON CPU performance
    let bandwidth = 50.0;

    (
        cpu_name,
        "CPU (SIMD Vectorized)".to_string(),
        "cpu".to_string(),
        is_apple,
        false,
        is_apple,
        0.0,
        tflops,
        bandwidth,
    )
}

/// Detect NVIDIA GPUs using nvidia-smi
fn detect_nvidia_gpu() -> Option<(String, f32, f32, f32)> {
    let output = Command::new("nvidia-smi")
        .args(["--query-gpu=name,memory.total", "--format=csv,noheader,nounits"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8(output.stdout).ok()?;
    let first_line = text.lines().next()?;
    let parts: Vec<&str> = first_line.split(',').collect();
    if parts.len() < 2 {
        return None;
    }

    let name = parts[0].trim().to_string();
    let vram_mb: f32 = parts[1].trim().parse().unwrap_or(8192.0);

    let (tflops, bandwidth) = estimate_nvidia_perf(&name);
    Some((name, vram_mb, tflops, bandwidth))
}

fn estimate_nvidia_perf(name: &str) -> (f32, f32) {
    // Returns (FP16 Tensor TFLOPS, Memory Bandwidth GB/s)
    let lower = name.to_lowercase();
    if lower.contains("h100") {
        (989.0, 3350.0)
    } else if lower.contains("a100") {
        (312.0, 2039.0)
    } else if lower.contains("4090") {
        (82.6, 1008.0)
    } else if lower.contains("4080") {
        (48.7, 716.8)
    } else if lower.contains("3090") {
        (35.6, 936.0)
    } else if lower.contains("3080") {
        (29.8, 760.0)
    } else if lower.contains("4070") {
        (29.0, 504.0)
    } else if lower.contains("3070") {
        (20.3, 448.0)
    } else if lower.contains("t4") {
        (65.0, 320.0)
    } else {
        (25.0, 400.0)
    }
}

/// Detect AMD GPUs using rocm-smi
fn detect_amd_gpu() -> Option<(String, f32, f32, f32)> {
    let output = Command::new("rocm-smi")
        .args(["--showproductname", "--showmeminfo", "vram"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8(output.stdout).ok()?;
    if text.contains("Radeon") || text.contains("Instinct") {
        Some(("AMD ROCm GPU".to_string(), 16384.0, 60.0, 800.0))
    } else {
        None
    }
}

#[cfg(target_os = "macos")]
fn estimate_apple_silicon_perf(brand: &str) -> (f32, f32) {
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
            "Detected Hardware: {} | GPU: {} (Backend: {}, UMA: {}, VRAM: {:.1} GB, TFLOPS: {:.1})",
            caps.device_name,
            caps.gpu_type,
            caps.backend,
            caps.unified_memory,
            caps.vram_gb,
            caps.estimated_tflops
        );
    }
}
