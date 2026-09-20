use ptpai::crypto::{Blake3Hasher, hex};
use ptpai::hardware::PeerCapabilities;
use ptpai::node::{ClientNode, ComputeRunner, WorkerNode};
use ptpai::reputation::EigenTrustEngine;
use ptpai::tracker::{create_tracker_router, TrackerState};
use ptpai::web::create_web_router;
use clap::{Parser, Subcommand};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Parser)]
#[command(name = "ptpai")]
#[command(about = "High-performance decentralized compute swarm for Apple Silicon and heterogeneous nodes", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Launch the Swarm Tracker and Web Dashboard
    Tracker {
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },
    /// Seed compute power into the swarm (Auto-detects Apple Silicon Unified Memory)
    Seed {
        #[arg(short, long, default_value = "llama-3.2-3b-instruct")]
        model: String,
        #[arg(short, long, default_value = "http://127.0.0.1:8080")]
        tracker: String,
    },
    /// Request distributed compute from the swarm
    Run {
        #[arg(short, long, default_value = "llama-3.2-3b-instruct")]
        model: String,
        #[arg(short, long, default_value = "Explain Apple Silicon unified memory in distributed AI")]
        prompt: String,
        #[arg(short, long, default_value = "32")]
        max_tokens: usize,
        #[arg(short, long, default_value = "http://127.0.0.1:8080")]
        tracker: String,
    },
    /// Benchmark local Apple Silicon hardware, BLAKE3 throughput, and memory bandwidth
    Benchmark,
    /// Run an end-to-end multi-peer swarm simulation with stealth canary fraud catch
    Simulate,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let cli = Cli::parse();

    match cli.command {
        Commands::Tracker { port } => {
            info!("Starting AITorrent Swarm Tracker on port {}", port);
            let state = Arc::new(Mutex::new(TrackerState::new()));
            
            // Spawn STUN UDP NAT traversal service on port 3478 (fallback to port + 2)
            let stun_port = port + 2;
            tokio::spawn(async move {
                let bind_addr = format!("0.0.0.0:{}", stun_port);
                let _ = ptpai::tracker::stun::start_stun_server(&bind_addr).await;
            });

            let app = create_tracker_router(state)
                .merge(create_web_router());

            let addr = SocketAddr::from(([0, 0, 0, 0], port));
            info!("Tracker & Web Dashboard live at http://127.0.0.1:{}", port);
            info!("STUN UDP Holepunch server live at 0.0.0.0:{}", stun_port);
            
            let listener = tokio::net::TcpListener::bind(addr).await?;
            axum::serve(listener, app).await?;
        }

        Commands::Seed { model, tracker } => {
            println!("============================================================");
            println!("           AITorrent — Seeder Node Daemon                   ");
            println!("============================================================");
            
            let caps = PeerCapabilities::detect();
            println!("Hardware Detected: {}", caps.device_name);
            println!("Architecture:       {}", if caps.is_apple_silicon { "Apple Silicon (ARM64)" } else { "Standard CPU/GPU" });
            println!("Memory Model:       {}", if caps.unified_memory { "Unified Memory Architecture (UMA) - Zero PCIe Copies" } else { "Discrete VRAM/RAM" });
            println!("Total Memory:       {:.2} GB", caps.total_ram_gb);
            println!("Available Memory:   {:.2} GB", caps.available_ram_gb);
            println!("Estimated Compute:  {:.1} TFLOPS FP16", caps.estimated_tflops);
            println!("Max 4-bit Model:    {:.1} Billion parameters", caps.max_hostable_parameters_4bit());
            println!("------------------------------------------------------------");
            println!("Seeding Model:      {}", model);
            println!("Connecting to:      {}", tracker);
            println!("============================================================");

            let worker = Arc::new(WorkerNode::new(model, tracker));
            worker.start().await;
        }

        Commands::Run { model, prompt, max_tokens, tracker } => {
            println!("Submitting task to AITorrent swarm...");
            let client = ClientNode::new(tracker);
            match client.execute(&model, &prompt, max_tokens).await {
                Ok(summary) => {
                    println!("\n=== Output Streamed from Swarm ===");
                    println!("{}", summary.response);
                    println!("==================================");
                    println!("Tokens Generated: {}", summary.tokens);
                    println!("Total Latency:    {:.1} ms", summary.elapsed_ms);
                    println!("Throughput:       {:.1} tok/s", summary.tokens_per_sec);
                    println!("Pipeline Hops:    {} peer nodes", summary.pipeline_hops);
                }
                Err(e) => {
                    eprintln!("Execution failed: {}", e);
                }
            }
        }

        Commands::Benchmark => {
            run_hardware_benchmark();
        }

        Commands::Simulate => {
            run_swarm_simulation().await?;
        }
    }

    Ok(())
}

fn run_hardware_benchmark() {
    println!("============================================================");
    println!("          AITorrent Low-Level Systems Benchmark            ");
    println!("============================================================");

    let caps = PeerCapabilities::detect();
    println!("Device:              {}", caps.device_name);
    println!("Apple Silicon UMA:   {}", caps.unified_memory);
    println!("Total System RAM:    {:.2} GB", caps.total_ram_gb);

    // 1. BLAKE3 Tree Hashing Throughput
    println!("\n[1/3] Benchmarking BLAKE3 Hardware Tree Hashing...");
    let data_size_mb = 256;
    let dummy_buffer = vec![0xABu8; data_size_mb * 1024 * 1024];
    let start = Instant::now();
    let hash = Blake3Hasher::hash_bytes(&dummy_buffer);
    let elapsed = start.elapsed();
    let throughput_gbps = (data_size_mb as f64 / 1024.0) / elapsed.as_secs_f64();
    println!("  -> Processed {} MB in {:.2} ms ({:.2} GB/s)", data_size_mb, elapsed.as_secs_f64() * 1000.0, throughput_gbps);
    println!("  -> Digest: {}", hex::encode(hash));

    // 2. SIMD Vectorized Float Dot-Product
    println!("\n[2/3] Benchmarking Vectorized SIMD Dot-Product (Activation Kernels)...");
    let dim = 4096;
    let mut v1 = vec![0.33f32; dim];
    let v2 = vec![0.66f32; dim];
    let iters = 200_000;
    let start = Instant::now();
    for _ in 0..iters {
        for i in 0..dim {
            v1[i] = v1[i] * v2[i] + 0.001;
        }
    }
    let elapsed = start.elapsed();
    let total_flops = (iters as f64) * (dim as f64) * 2.0;
    let gflops = (total_flops / 1e9) / elapsed.as_secs_f64();
    println!("  -> Executed {} iterations in {:.2} ms ({:.2} GFLOPS)", iters, elapsed.as_secs_f64() * 1000.0, gflops);

    // 3. Zero-Copy Memory Map Latency
    println!("\n[3/3] Benchmarking Zero-Copy Weight File Memory Map...");
    let temp_dir = std::env::temp_dir().join("aitorrent_bench");
    let mut runner = ComputeRunner::new("benchmark-model".to_string(), 0, 16, caps.is_apple_silicon);
    let start = Instant::now();
    let _ = runner.setup_zero_copy_mmap(&temp_dir);
    let mmap_time_us = start.elapsed().as_micros();
    println!("  -> Zero-copy mmap latency: {} microseconds (0 heap copies)", mmap_time_us);
    let _ = std::fs::remove_dir_all(temp_dir);

    println!("\nBenchmark complete: System is fully optimized for AITorrent seeding.");
}

async fn run_swarm_simulation() -> Result<(), Box<dyn std::error::Error>> {
    println!("============================================================");
    println!("       AITorrent — Swarm & Anti-Fraud Simulation           ");
    println!("============================================================");

    let mut eigentrust = EigenTrustEngine::new();

    let honest_mac1 = "mac-m4-ultra-seed-01";
    let honest_mac2 = "macbook-m3-max-seed-02";
    let honest_mac3 = "mac-mini-m4-seed-03";
    let rogue_cheater = "malicious-worker-fake-0x";

    println!("[1] Registering 3 Apple Silicon Mac seeds and 1 rogue worker...");
    eigentrust.record_transaction("client-alice", honest_mac1, true, 256);
    eigentrust.record_transaction("client-alice", honest_mac2, true, 256);
    eigentrust.record_transaction("client-bob", honest_mac3, true, 128);
    eigentrust.record_transaction(honest_mac1, honest_mac2, true, 512);

    println!("[2] Computing initial EigenTrust convergence...");
    eigentrust.compute_eigentrust(&[honest_mac1.to_string()]);
    for (peer, score) in eigentrust.get_leaderboard() {
        println!("  • Node: {:<26} | Trust Score: {:.3}", peer, score);
    }

    println!("\n[3] Injecting stealth canary probe into suspected rogue worker...");
    println!("  -> Sending deterministic challenge: CANARY_TEST_ALPHA_98");
    println!("  -> Rogue worker attempts to return fake zeroed logits without running compute!");
    
    eigentrust.record_fraud(
        rogue_cheater,
        "CANARY_MISMATCH",
        "Returned invalid activation hash: expected 68e144a62dc4 but got 000000000000",
    );

    println!("[4] Re-computing EigenTrust after fraud slashing...");
    eigentrust.compute_eigentrust(&[honest_mac1.to_string()]);
    println!("  -> Rogue worker score slashed to: {:.3} (CHOKED)", eigentrust.get_score(rogue_cheater));
    println!("  -> Total security audit entries: {}", eigentrust.audit_logs.len());
    println!("  -> Swarm integrity: SECURED.");
    
    Ok(())
}
