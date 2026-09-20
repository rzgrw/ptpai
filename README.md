# PTPAI (⚡) — Peer-to-Peer AI Compute

> High-performance, zero-copy decentralized compute swarm engine written in **Rust**, optimized for **Apple Silicon Unified Memory (UMA)**, WebGPU browser workers, and heterogeneous compute nodes.

---

## 💡 Why Apple Silicon Unified Memory?

Standard distributed compute across discrete GPUs (NVIDIA PCIe) incurs massive memory copying bottlenecks between host system RAM and discrete GPU VRAM.

On Apple Silicon (M1/M2/M3/M4, Pro, Max, Ultra):
- **CPU, GPU, and Neural Engine share a unified physical memory pool.**
- Memory bandwidth reaches **100 GB/s to 800+ GB/s**.
- Using **zero-copy memory mapping (`memmap2`)**, verified BitTorrent model shards are mapped directly into the address space with **0 PCIe copies and 0 heap allocations**.
- Mac users with 16GB, 24GB, 36GB, 64GB, or 128GB unified memory can seed model layers collaboratively with zero setup friction.

---

## ⚡ Low-Level Systems Architecture

1. **Zero-Copy Wire Protocol (`ptpai::wire`)**:
   - Fixed 32-byte binary header with `b"AIT1"` magic, flags, monotonic sequence ID, and truncated BLAKE3 checksums.
   - $O(1)$ memory framing over Tokio async streams.
2. **BLAKE3 Hardware Hashing (`ptpai::crypto`)**:
   - Multithreaded SIMD NEON tree hashing saturating up to **2+ GB/s per core**.
   - Cryptographic Ed25519 node identities and signed `ComputeReceipt`s.
3. **Decentralized Reputation & Anti-Fraud (`ptpai::reputation`)**:
   - **Compute Tit-for-Tat (cT4T)**: Accounting ledger for tokens served vs consumed; freeloaders are choked.
   - **EigenTrust Convergence**: Solves global trust scores using power iteration over peer transaction reviews.
   - **Stealth Canaries**: Randomly injected challenges that catch lazy or cheating nodes returning fake logits, immediately slashing their score to 0.0 and choking them.
4. **qBittorrent Web Client & Model Hub (`ptpai::web`)**:
   - Real-time qBittorrent-inspired WebUI with 112-piece layer visualizer and live token throughput graphs.
   - In-browser **WebGPU Metal** compute engine allowing anyone to seed by simply opening a URL.
   - Delta synchronization engine matching qBittorrent's `/api/v2/sync/maindata` protocol.

---

## 🚀 Quickstart for Mac Users

### 1. Build Optimized Release Binary

```bash
cargo build --release
```

### 2. Run Local Systems Benchmark

Measures your Mac's unified memory bandwidth, BLAKE3 tree hashing speed, and SIMD activation kernels:

```bash
./target/release/ptpai benchmark
```

### 3. Start Swarm Tracker & Web Dashboard

```bash
./target/release/ptpai tracker --port 8080
```
Open **`http://127.0.0.1:8080`** to view the live dashboard, peer topology, and token streaming playground.

### 4. 1-Command Compute Seeding

Auto-detects your Apple Silicon chip, unified memory capacity, and starts seeding:

```bash
./target/release/ptpai seed --model llama-3.2-3b-instruct
```

### 5. Request Distributed Compute

Run a query routed across peer Mac seeds:

```bash
./target/release/ptpai run --prompt "Explain how torrent swarms accelerate AI"
```

### 6. Run Swarm & Anti-Fraud Simulation

Demonstrates EigenTrust convergence, stealth canary probes, and fraud choking:

```bash
./target/release/ptpai simulate
```
