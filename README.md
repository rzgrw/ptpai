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

## 🚀 Quickstart: 1-Line Universal Installer (macOS & Ubuntu)

On **macOS** (Apple Silicon M1/M2/M3/M4) or **Ubuntu Linux** (NVIDIA CUDA / x86_64), install with a single command:

```bash
curl -fsSL https://raw.githubusercontent.com/rzgrw/ptpai/main/install.sh | bash
```

Or build manually from source:
```bash
git clone https://github.com/rzgrw/ptpai.git
cd ptpai
cargo build --release
```

---

## 🍏 macOS Usage (Apple Silicon & Metal)

1. **Benchmark Hardware & Unified Memory Bandwidth**:
   ```bash
   ptpai benchmark
   ```
2. **Start Swarm Tracker & qBittorrent WebUI**:
   ```bash
   ptpai tracker --port 8080
   ```
   Open **`http://127.0.0.1:8080`** to browse models, copy Magnet links, and seed via WebGPU.
3. **1-Command Seeding (Auto-Detects Unified Memory & Metal)**:
   ```bash
   ptpai seed --model llama-3.2-3b-instruct
   ```

---

## 🐧 Ubuntu Linux Usage (NVIDIA CUDA & Server Daemons)

1. **Native Seeding on Ubuntu with NVIDIA GPU**:
   Auto-detects `nvidia-smi`, VRAM size, and starts seeding:
   ```bash
   ptpai seed --model llama-3.2-3b-instruct --tracker http://<tracker-ip>:8080
   ```

2. **Bridge an Existing Local Ollama / vLLM / llama.cpp Server**:
   ```bash
   ptpai seed \
     --model llama-3.2-3b-instruct \
     --tracker http://<tracker-ip>:8080 \
     --bridge http://127.0.0.1:11434
   ```

3. **Run via Docker with NVIDIA Container Toolkit**:
   ```bash
   # Build the container
   docker build -t ptpai:latest .

   # Run with full NVIDIA GPU passthrough
   docker run -d --gpus all \
     -p 8080:8080 -p 8082:8082/udp \
     --restart unless-stopped \
     --name ptpai-daemon \
     ptpai:latest tracker --port 8080
   ```

4. **Run as a 24/7 Ubuntu Systemd Service**:
   ```bash
   sudo cp ptpai.service /etc/systemd/system/
   sudo systemctl daemon-reload
   sudo systemctl enable --now ptpai
   ```

---

## 💬 Request Distributed Compute from the Swarm

Run a prompt routed across the heterogeneous Mac + Ubuntu pipeline:
```bash
ptpai run --prompt "Explain how P2P compute pipelines cross-operate between Apple Silicon and Linux NVIDIA GPUs"
```
