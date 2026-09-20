use crate::crypto::{ComputeReceipt, NodeIdentity};
use crate::hardware::PeerCapabilities;
use crate::node::runner::ComputeRunner;
use crate::tracker::AnnounceRequest;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{info, warn};

pub struct WorkerNode {
    pub identity: NodeIdentity,
    pub capabilities: PeerCapabilities,
    pub model_id: String,
    pub layer_range: (u32, u32),
    pub tracker_url: String,
    pub bridge_url: Option<String>,
    pub runner: ComputeRunner,
}

impl WorkerNode {
    pub fn new(model_id: String, tracker_url: String, bridge_url: Option<String>) -> Self {
        let identity = NodeIdentity::generate();
        let capabilities = PeerCapabilities::detect();
        let is_apple = capabilities.is_apple_silicon;

        // Auto-assign layer range based on available unified memory or GPU VRAM
        let layer_range = if capabilities.total_ram_gb >= 32.0 || capabilities.vram_gb >= 24.0 {
            (0, 28) // Host entire model in unified memory / GPU VRAM
        } else if capabilities.total_ram_gb >= 16.0 || capabilities.vram_gb >= 8.0 {
            (0, 14) // Host 14-layer pipeline shard
        } else {
            (0, 7)  // Host 7-layer slice on smaller GPUs
        };

        let mut runner = ComputeRunner::new(
            model_id.clone(),
            layer_range.0,
            layer_range.1,
            is_apple,
        );

        let cache_dir = PathBuf::from("./.aitorrent_cache");
        let _ = runner.setup_zero_copy_mmap(&cache_dir);

        Self {
            identity,
            capabilities,
            model_id,
            layer_range,
            tracker_url,
            bridge_url,
            runner,
        }
    }

    /// Announce node to tracker and register hardware capabilities
    pub async fn announce(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest_or_hyper_announce(
            &self.tracker_url,
            &AnnounceRequest {
                peer_id: self.identity.peer_id().to_string(),
                address: "127.0.0.1:9092".to_string(),
                capabilities: self.capabilities.clone(),
                seeded_models: vec![self.model_id.clone()],
                layer_range: self.layer_range,
            },
        ).await?;

        info!(
            "Successfully announced to tracker at {}. Status: {}",
            self.tracker_url, client
        );
        Ok(())
    }

    /// Execute a compute request and sign the receipt
    pub fn execute_job(
        &self,
        task_id: &str,
        consumer_peer_id: &str,
        prompt: &str,
        max_tokens: usize,
    ) -> (String, ComputeReceipt) {
        let result = self.runner.execute_prompt(prompt, max_tokens);

        let receipt = ComputeReceipt::new_signed(
            &self.identity,
            task_id.to_string(),
            consumer_peer_id.to_string(),
            self.model_id.clone(),
            result.tokens_generated,
            (self.capabilities.estimated_tflops as f64) * (result.latency_ms / 1000.0) * 0.1,
            result.latency_ms,
        );

        (result.output_text, receipt)
    }

    /// Start the worker loop: announces and maintains heartbeats
    pub async fn start(self: Arc<Self>) {
        info!("Starting PTPAI Worker Daemon");
        info!("Peer ID: {}", self.identity.peer_id());
        info!(
            "Hardware: {} | GPU: {} (Backend: {})",
            self.capabilities.device_name, self.capabilities.gpu_type, self.capabilities.backend
        );
        info!(
            "Memory Pool: Total RAM: {:.1} GB | VRAM: {:.1} GB | TFLOPS: {:.1}",
            self.capabilities.total_ram_gb, self.capabilities.vram_gb, self.capabilities.estimated_tflops
        );
        if let Some(ref b) = self.bridge_url {
            info!("Connected to Local Linux GPU Bridge: {}", b);
        }
        info!(
            "Seeding Model: {} (Layers {}-{}) via Zero-Copy mmap",
            self.model_id, self.layer_range.0, self.layer_range.1
        );

        loop {
            if let Err(e) = self.announce().await {
                warn!("Tracker announce failed: {}. Retrying in 5s...", e);
            }
            sleep(Duration::from_secs(10)).await;
        }
    }
}

async fn reqwest_or_hyper_announce(
    tracker_url: &str,
    req: &AnnounceRequest,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let url = format!("{}/api/announce", tracker_url);
    let client = reqwest::Client::new();
    let resp = client.post(&url).json(req).send().await?;
    let status = resp.status();
    Ok(format!("{}", status))
}
