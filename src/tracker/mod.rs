pub mod stun;

use crate::crypto::ComputeReceipt;
use crate::hardware::PeerCapabilities;
use crate::reputation::{EigenTrustEngine, PeerRatioRecord, SecurityAuditLog};
use axum::{
    extract::{
        Query, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub peer_id: String,
    pub address: String,
    pub capabilities: PeerCapabilities,
    pub seeded_models: Vec<String>,
    pub layer_range: (u32, u32),
    pub reputation_score: f64,
    pub ratio: f64,
    pub is_choked: bool,
    pub total_tokens_served: u64,
    pub last_seen_secs_ago: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnounceRequest {
    pub peer_id: String,
    pub address: String,
    pub capabilities: PeerCapabilities,
    pub seeded_models: Vec<String>,
    pub layer_range: (u32, u32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmStats {
    pub total_peers: usize,
    pub apple_silicon_peers: usize,
    pub total_unified_memory_gb: f32,
    pub total_swarm_tflops: f32,
    pub active_models: Vec<String>,
    pub total_tokens_processed: u64,
    pub total_receipts_verified: u64,
    pub cheated_nodes_choked: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteRequest {
    pub model_id: String,
    pub prompt: String,
    pub max_tokens: usize,
    pub client_peer_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineHop {
    pub peer_id: String,
    pub device_name: String,
    pub is_apple_silicon: bool,
    pub layer_start: u32,
    pub layer_end: u32,
    pub reputation_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteResponse {
    pub pipeline: Vec<PipelineHop>,
    pub total_estimated_latency_ms: f64,
    pub is_routed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogEntry {
    pub model_id: String,
    pub title: String,
    pub description: String,
    pub parameter_size_b: f32,
    pub quantization: String,
    pub total_layers: u32,
    pub size_gb: f32,
    pub required_unified_ram_gb: f32,
    pub magnet_uri: String,
    pub info_hash: String,
    pub active_seeds: usize,
    pub active_peers: usize,
    pub layer_coverage_pct: f32,
    pub tokens_computed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmEvent {
    pub event_type: String,
    pub peer_id: String,
    pub message: String,
    pub timestamp_epoch: i64,
}

pub struct TrackerState {
    pub peers: HashMap<String, (PeerInfo, std::time::Instant)>,
    pub ratios: HashMap<String, PeerRatioRecord>,
    pub eigentrust: EigenTrustEngine,
    pub receipts: Vec<ComputeReceipt>,
    pub catalog: Vec<CatalogEntry>,
    pub rid: u64,
    pub event_tx: broadcast::Sender<SwarmEvent>,
}

#[derive(Debug, Deserialize)]
pub struct SyncQueryParams {
    pub rid: Option<u64>,
}

impl TrackerState {
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(1000);
        let mut state = Self {
            peers: HashMap::new(),
            ratios: HashMap::new(),
            eigentrust: EigenTrustEngine::new(),
            receipts: Vec::new(),
            catalog: Vec::new(),
            rid: 1,
            event_tx,
        };
        state.init_default_catalog();
        state.seed_initial_mac_peers();
        state
    }

    fn init_default_catalog(&mut self) {
        self.catalog = vec![
            CatalogEntry {
                model_id: "llama-3.2-3b-instruct".to_string(),
                title: "Meta Llama 3.2 3B Instruct".to_string(),
                description: "Optimized for 2x M4 Mac Airs (14 layers per machine). Ultra fast reasoning, fanless low-power compute.".to_string(),
                parameter_size_b: 3.21,
                quantization: "q4_k_m".to_string(),
                total_layers: 28,
                size_gb: 1.82,
                required_unified_ram_gb: 0.95,
                magnet_uri: "magnet:?xt=urn:ait:2b6c01160e41a44187403b9ce4a1e12e&dn=llama-3.2-3b-instruct&tr=http://127.0.0.1:8080/api/announce&xl=1953504256&layers=28".to_string(),
                info_hash: "2b6c01160e41a44187403b9ce4a1e12e952c5729".to_string(),
                active_seeds: 2,
                active_peers: 2,
                layer_coverage_pct: 100.0,
                tokens_computed: 12600,
            },
            CatalogEntry {
                model_id: "deepseek-r1-distill-1.5b".to_string(),
                title: "DeepSeek R1 Distill Qwen 1.5B".to_string(),
                description: "Lightweight reasoning model with chain-of-thought tokens. Minimal 500 MB RAM slice per peer.".to_string(),
                parameter_size_b: 1.54,
                quantization: "q4_k_m".to_string(),
                total_layers: 28,
                size_gb: 1.05,
                required_unified_ram_gb: 0.55,
                magnet_uri: "magnet:?xt=urn:ait:9a7bc01ef8324567890abcdef1234567&dn=deepseek-r1-distill-1.5b&tr=http://127.0.0.1:8080/api/announce&xl=1127219200&layers=28".to_string(),
                info_hash: "9a7bc01ef8324567890abcdef1234567890abcde".to_string(),
                active_seeds: 2,
                active_peers: 2,
                layer_coverage_pct: 100.0,
                tokens_computed: 8400,
            },
            CatalogEntry {
                model_id: "deepseek-r1-distill-8b".to_string(),
                title: "DeepSeek R1 Distill Llama 8B".to_string(),
                description: "Flagship open reasoning model. Split across 4 Mac Airs or 2 Mac Studios.".to_string(),
                parameter_size_b: 8.03,
                quantization: "q4_k_m".to_string(),
                total_layers: 32,
                size_gb: 4.80,
                required_unified_ram_gb: 2.40,
                magnet_uri: "magnet:?xt=urn:ait:68e144a62dc4567890abcdef1234567&dn=deepseek-r1-distill-8b&tr=http://127.0.0.1:8080/api/announce&xl=5153960755&layers=32".to_string(),
                info_hash: "68e144a62dc4567890abcdef1234567890abcdef".to_string(),
                active_seeds: 1,
                active_peers: 3,
                layer_coverage_pct: 50.0,
                tokens_computed: 23100,
            },
        ];
    }

    fn seed_initial_mac_peers(&mut self) {
        let now = std::time::Instant::now();
        let demo_nodes = vec![
            (
                "04a8b7c6d5e4f3a2b1c0d9e8f7a6b5c4d3e2f1a0b9c8d7e6f5a4b3c2d1e0f9a8",
                "MacBook Air #1 (Apple M4 / 16 GB UMA)",
                true,
                16.0,
                38.0,
                vec![
                    "llama-3.2-3b-instruct".to_string(),
                    "deepseek-r1-distill-1.5b".to_string(),
                ],
                (0, 14),
            ),
            (
                "15b9c8d7e6f5a4b3c2d1e0f9a8b7c6d5e4f3a2b1c0d9e8f7a6b5c4d3e2f1a0b9",
                "MacBook Air #2 (Apple M4 / 16 GB UMA)",
                true,
                16.0,
                38.0,
                vec![
                    "llama-3.2-3b-instruct".to_string(),
                    "deepseek-r1-distill-1.5b".to_string(),
                ],
                (14, 28),
            ),
        ];

        for (id, dev, is_apple, ram, tflops, models, layers) in demo_nodes {
            let caps = PeerCapabilities {
                device_name: dev.to_string(),
                gpu_type: "Apple Silicon GPU (Metal)".to_string(),
                backend: "metal".to_string(),
                is_apple_silicon: is_apple,
                is_discrete_gpu: false,
                unified_memory: is_apple,
                total_ram_gb: ram,
                available_ram_gb: ram * 0.8,
                vram_gb: ram,
                cpu_cores: 10,
                estimated_tflops: tflops,
                memory_bandwidth_gbps: 120.0,
                supported_quantizations: vec!["q4_k_m".to_string(), "fp8".to_string()],
            };

            let info = PeerInfo {
                peer_id: id.to_string(),
                address: "127.0.0.1:9091".to_string(),
                capabilities: caps,
                seeded_models: models,
                layer_range: layers,
                reputation_score: 0.95,
                ratio: 1.42,
                is_choked: false,
                total_tokens_served: 4200,
                last_seen_secs_ago: 0,
            };

            self.peers.insert(id.to_string(), (info, now));
            let mut record = PeerRatioRecord::new(id.to_string());
            record.tokens_served = 4200;
            self.ratios.insert(id.to_string(), record);
        }
    }

    pub fn get_stats(&self) -> SwarmStats {
        let mut total_ram = 0.0f32;
        let mut total_tflops = 0.0f32;
        let mut apple_peers = 0;
        let mut models = std::collections::HashSet::new();
        let mut total_tokens = 0u64;

        for (info, _) in self.peers.values() {
            total_ram += info.capabilities.total_ram_gb;
            total_tflops += info.capabilities.estimated_tflops;
            if info.capabilities.is_apple_silicon {
                apple_peers += 1;
            }
            for m in &info.seeded_models {
                models.insert(m.clone());
            }
            total_tokens += info.total_tokens_served;
        }

        let choked_count = self.peers.values().filter(|(p, _)| p.is_choked).count();

        SwarmStats {
            total_peers: self.peers.len(),
            apple_silicon_peers: apple_peers,
            total_unified_memory_gb: total_ram,
            total_swarm_tflops: total_tflops,
            active_models: models.into_iter().collect(),
            total_tokens_processed: total_tokens,
            total_receipts_verified: self.receipts.len() as u64,
            cheated_nodes_choked: choked_count,
        }
    }

    /// Automatically prune stale peers that stopped heartbeating (> 45s idle)
    pub fn prune_stale_peers(&mut self, max_idle_secs: u64) {
        let now = std::time::Instant::now();
        let stale_ids: Vec<String> = self
            .peers
            .iter()
            .filter(|(id, (_, instant))| {
                !id.starts_with("04a8b") && !id.starts_with("15b9c") && now.duration_since(*instant).as_secs() > max_idle_secs
            })
            .map(|(id, _)| id.clone())
            .collect();

        for id in stale_ids {
            self.peers.remove(&id);
        }
    }
}

pub type SharedTrackerState = Arc<RwLock<TrackerState>>;

pub fn create_tracker_router(state: SharedTrackerState) -> Router {
    Router::new()
        .route("/api/status", get(handle_status))
        .route("/api/peers", get(handle_peers))
        .route("/api/catalog", get(handle_catalog))
        .route("/api/catalog/publish", post(handle_publish_catalog))
        .route("/api/v2/sync/maindata", get(handle_sync_maindata))
        .route("/api/announce", post(handle_announce))
        .route("/api/peer/leave", post(handle_peer_leave))
        .route("/api/route", post(handle_route))
        .route("/api/receipt", post(handle_receipt))
        .route("/api/audit", get(handle_audit))
        .route("/api/simulate-fraud", post(handle_simulate_fraud))
        .route("/ws/telemetry", get(handle_ws_telemetry))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

#[derive(Debug, Deserialize)]
pub struct LeaveRequest {
    pub peer_id: String,
}

async fn handle_peer_leave(
    State(state): State<SharedTrackerState>,
    Json(payload): Json<LeaveRequest>,
) -> Json<serde_json::Value> {
    let mut s = state.write();
    s.peers.remove(&payload.peer_id);
    let _ = s.event_tx.send(SwarmEvent {
        event_type: "PEER_LEFT".to_string(),
        peer_id: payload.peer_id.clone(),
        message: format!("Peer gracefully disconnected from swarm: {}", payload.peer_id),
        timestamp_epoch: chrono::Utc::now().timestamp(),
    });
    Json(serde_json::json!({ "status": "removed", "peer_id": payload.peer_id }))
}

async fn handle_sync_maindata(
    State(state): State<SharedTrackerState>,
    Query(params): Query<SyncQueryParams>,
) -> Json<serde_json::Value> {
    let mut s = state.write();
    s.prune_stale_peers(45);
    s.rid += 1;
    let current_rid = s.rid;
    let client_rid = params.rid.unwrap_or(0);
    let is_full = client_rid == 0;

    let stats = s.get_stats();

    let mut torrents_map = serde_json::Map::new();
    for entry in &s.catalog {
        let mut t_obj = serde_json::Map::new();
        t_obj.insert("name".to_string(), serde_json::json!(entry.title));
        t_obj.insert("size".to_string(), serde_json::json!((entry.size_gb * 1024.0 * 1024.0 * 1024.0) as u64));
        t_obj.insert("progress".to_string(), serde_json::json!(entry.layer_coverage_pct / 100.0));
        t_obj.insert("state".to_string(), serde_json::json!("seeding"));
        t_obj.insert("num_seeds".to_string(), serde_json::json!(entry.active_seeds));
        t_obj.insert("num_leechs".to_string(), serde_json::json!(entry.active_peers));
        t_obj.insert("dlspeed".to_string(), serde_json::json!(0));
        t_obj.insert("upspeed".to_string(), serde_json::json!(84200));
        t_obj.insert("ratio".to_string(), serde_json::json!(1.42));
        t_obj.insert("magnet_uri".to_string(), serde_json::json!(entry.magnet_uri));
        t_obj.insert("total_layers".to_string(), serde_json::json!(entry.total_layers));
        t_obj.insert("required_ram".to_string(), serde_json::json!(entry.required_unified_ram_gb));
        torrents_map.insert(entry.info_hash.clone(), serde_json::Value::Object(t_obj));
    }

    let mut peers_map = serde_json::Map::new();
    let now = std::time::Instant::now();
    for (info, instant) in s.peers.values() {
        let mut p_obj = serde_json::Map::new();
        p_obj.insert("client".to_string(), serde_json::json!(info.capabilities.device_name));
        p_obj.insert("ip".to_string(), serde_json::json!(info.address));
        p_obj.insert("ram_gb".to_string(), serde_json::json!(info.capabilities.total_ram_gb));
        p_obj.insert("tflops".to_string(), serde_json::json!(info.capabilities.estimated_tflops));
        p_obj.insert("hosted_layers".to_string(), serde_json::json!(vec![info.layer_range.0, info.layer_range.1]));
        p_obj.insert("reputation".to_string(), serde_json::json!(info.reputation_score));
        p_obj.insert("choked".to_string(), serde_json::json!(info.is_choked));
        p_obj.insert("last_seen".to_string(), serde_json::json!(now.duration_since(*instant).as_secs()));
        peers_map.insert(info.peer_id.clone(), serde_json::Value::Object(p_obj));
    }

    let server_state = serde_json::json!({
        "dl_info_speed": 0,
        "up_info_speed": 92400,
        "total_peers": stats.total_peers,
        "total_unified_memory_gb": stats.total_unified_memory_gb,
        "total_swarm_tflops": stats.total_swarm_tflops,
        "connection_status": "connected",
        "dht_nodes": s.peers.len(),
    });

    Json(serde_json::json!({
        "rid": current_rid,
        "full_update": is_full,
        "torrents": torrents_map,
        "peers": peers_map,
        "server_state": server_state,
    }))
}

async fn handle_catalog(State(state): State<SharedTrackerState>) -> Json<Vec<CatalogEntry>> {
    let s = state.read();
    Json(s.catalog.clone())
}

async fn handle_publish_catalog(
    State(state): State<SharedTrackerState>,
    Json(entry): Json<CatalogEntry>,
) -> Json<serde_json::Value> {
    let mut s = state.write();
    s.catalog.push(entry.clone());
    let _ = s.event_tx.send(SwarmEvent {
        event_type: "TORRENT_PUBLISHED".to_string(),
        peer_id: "catalog".to_string(),
        message: format!("New compute torrent published: {}", entry.title),
        timestamp_epoch: chrono::Utc::now().timestamp(),
    });
    Json(serde_json::json!({
        "status": "ok",
        "model_id": entry.model_id,
        "magnet_uri": entry.magnet_uri,
    }))
}

async fn handle_status(State(state): State<SharedTrackerState>) -> Json<SwarmStats> {
    let mut s = state.write();
    s.prune_stale_peers(45);
    Json(s.get_stats())
}

async fn handle_peers(State(state): State<SharedTrackerState>) -> Json<Vec<PeerInfo>> {
    let mut s = state.write();
    s.prune_stale_peers(45);
    let mut list = Vec::new();
    let now = std::time::Instant::now();

    for (info, instant) in s.peers.values() {
        let mut copy = info.clone();
        copy.last_seen_secs_ago = now.duration_since(*instant).as_secs();
        list.push(copy);
    }
    Json(list)
}

async fn handle_announce(
    State(state): State<SharedTrackerState>,
    Json(payload): Json<AnnounceRequest>,
) -> Json<serde_json::Value> {
    let mut s = state.write();
    let now = std::time::Instant::now();
    let peer_id = payload.peer_id.clone();

    let rep_score = s.eigentrust.get_score(&peer_id);
    let (ratio, is_choked, tokens_served) = {
        let ratio_rec = s
            .ratios
            .entry(peer_id.clone())
            .or_insert_with(|| PeerRatioRecord::new(peer_id.clone()));
        (ratio_rec.ratio(), ratio_rec.is_choked, ratio_rec.tokens_served)
    };

    let info = PeerInfo {
        peer_id: peer_id.clone(),
        address: payload.address,
        capabilities: payload.capabilities.clone(),
        seeded_models: payload.seeded_models,
        layer_range: payload.layer_range,
        reputation_score: rep_score,
        ratio,
        is_choked,
        total_tokens_served: tokens_served,
        last_seen_secs_ago: 0,
    };

    let is_new = !s.peers.contains_key(&peer_id);
    s.peers.insert(peer_id.clone(), (info, now));

    if is_new {
        let _ = s.event_tx.send(SwarmEvent {
            event_type: "PEER_JOINED".to_string(),
            peer_id: peer_id.clone(),
            message: format!(
                "Peer joined swarm: {} (GPU: {}, VRAM: {:.1} GB, TFLOPS: {:.1})",
                payload.capabilities.device_name,
                payload.capabilities.gpu_type,
                payload.capabilities.vram_gb,
                payload.capabilities.estimated_tflops
            ),
            timestamp_epoch: chrono::Utc::now().timestamp(),
        });
    }

    Json(serde_json::json!({
        "status": "ok",
        "peer_id": peer_id,
        "is_choked": is_choked,
        "reputation_score": rep_score,
    }))
}

async fn handle_route(
    State(state): State<SharedTrackerState>,
    Json(payload): Json<RouteRequest>,
) -> Json<RouteResponse> {
    let s = state.read();
    let mut candidate_peers: Vec<PeerInfo> = s
        .peers
        .values()
        .map(|(p, _)| p.clone())
        .filter(|p| !p.is_choked && p.seeded_models.contains(&payload.model_id))
        .collect();

    candidate_peers.sort_by(|a, b| {
        a.layer_range.0.cmp(&b.layer_range.0).then_with(|| {
            b.reputation_score
                .partial_cmp(&a.reputation_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    });

    let mut pipeline = Vec::new();
    let mut covered_layers = 0u32;
    let target_layers = match payload.model_id.as_str() {
        "llama-3.2-3b-instruct" | "deepseek-r1-distill-1.5b" => 28u32,
        _ => 32u32,
    };

    for peer in candidate_peers {
        if covered_layers >= target_layers {
            break;
        }
        if peer.layer_range.0 <= covered_layers && peer.layer_range.1 > covered_layers {
            let hop = PipelineHop {
                peer_id: peer.peer_id.clone(),
                device_name: peer.capabilities.device_name.clone(),
                is_apple_silicon: peer.capabilities.is_apple_silicon,
                layer_start: peer.layer_range.0,
                layer_end: peer.layer_range.1,
                reputation_score: peer.reputation_score,
            };
            covered_layers = peer.layer_range.1;
            pipeline.push(hop);
        }
    }

    let is_routed = covered_layers >= target_layers && !pipeline.is_empty();
    let est_latency = if is_routed {
        (pipeline.len() as f64) * 22.5 + 15.0
    } else {
        0.0
    };

    Json(RouteResponse {
        pipeline,
        total_estimated_latency_ms: est_latency,
        is_routed,
    })
}

async fn handle_receipt(
    State(state): State<SharedTrackerState>,
    Json(receipt): Json<ComputeReceipt>,
) -> Json<serde_json::Value> {
    let mut s = state.write();
    if !receipt.verify() {
        return Json(serde_json::json!({
            "status": "error",
            "error": "Invalid cryptographic Ed25519 signature on compute receipt",
        }));
    }

    let provider = receipt.provider_peer_id.clone();
    let consumer = receipt.consumer_peer_id.clone();
    let tokens = receipt.tokens_processed;

    let provider_ratio = {
        let prov_rec = s
            .ratios
            .entry(provider.clone())
            .or_insert_with(|| PeerRatioRecord::new(provider.clone()));
        prov_rec.tokens_served += tokens;
        prov_rec.tflops_served += receipt.estimated_tflops;
        prov_rec.successful_jobs += 1;
        prov_rec.update_choke_status();
        prov_rec.ratio()
    };

    {
        let cons_rec = s
            .ratios
            .entry(consumer.clone())
            .or_insert_with(|| PeerRatioRecord::new(consumer.clone()));
        cons_rec.tokens_consumed += tokens;
        cons_rec.tflops_consumed += receipt.estimated_tflops;
        cons_rec.update_choke_status();
    }

    s.eigentrust
        .record_transaction(&consumer, &provider, true, tokens);
    s.eigentrust.compute_eigentrust(&[]);

    let _ = s.event_tx.send(SwarmEvent {
        event_type: "RECEIPT_VERIFIED".to_string(),
        peer_id: provider.clone(),
        message: format!(
            "Verified receipt: {} tokens computed (Latency: {:.1} ms)",
            tokens, receipt.latency_ms
        ),
        timestamp_epoch: chrono::Utc::now().timestamp(),
    });

    s.receipts.push(receipt);

    Json(serde_json::json!({
        "status": "ok",
        "verified": true,
        "provider_ratio": provider_ratio,
    }))
}

async fn handle_audit(State(state): State<SharedTrackerState>) -> Json<Vec<SecurityAuditLog>> {
    let s = state.read();
    Json(s.eigentrust.audit_logs.clone())
}

async fn handle_simulate_fraud(
    State(state): State<SharedTrackerState>,
) -> Json<serde_json::Value> {
    let mut s = state.write();
    let cheater_id = "badf00d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f7a8b9";

    s.eigentrust.record_fraud(
        cheater_id,
        "CANARY_MISMATCH",
        "Peer failed stealth canary verification: returned zeroed activation tensor instead of genuine transformer weights pass",
    );

    let rec = s
        .ratios
        .entry(cheater_id.to_string())
        .or_insert_with(|| PeerRatioRecord::new(cheater_id.to_string()));
    rec.failed_or_cheated_jobs += 1;
    rec.is_choked = true;

    if let Some((info, _)) = s.peers.get_mut(cheater_id) {
        info.is_choked = true;
        info.reputation_score = 0.0;
    }

    let _ = s.event_tx.send(SwarmEvent {
        event_type: "FRAUD_CHOKED".to_string(),
        peer_id: cheater_id.to_string(),
        message: "ALERT: Malicious worker caught by stealth canary! EigenTrust rating slashed to 0.0, node choked by swarm.".to_string(),
        timestamp_epoch: chrono::Utc::now().timestamp(),
    });

    Json(serde_json::json!({
        "status": "cheater_choked",
        "peer_id": cheater_id,
        "action": "slashed_and_choked",
    }))
}

async fn handle_ws_telemetry(
    ws: WebSocketUpgrade,
    State(state): State<SharedTrackerState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| telemetry_socket(socket, state))
}

async fn telemetry_socket(mut socket: WebSocket, state: SharedTrackerState) {
    let mut rx = {
        let s = state.read();
        s.event_tx.subscribe()
    };

    while let Ok(event) = rx.recv().await {
        if let Ok(json_str) = serde_json::to_string(&event) {
            if socket.send(Message::Text(json_str.into())).await.is_err() {
                break;
            }
        }
    }
}
