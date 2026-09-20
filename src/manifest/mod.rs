use crate::crypto::{Blake3Hasher, hex};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelPiece {
    pub piece_index: u32,
    pub layer_start: u32,
    pub layer_end: u32,
    pub size_bytes: u64,
    pub blake3_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AiTorrentManifest {
    pub model_id: String,
    pub display_name: String,
    pub parameter_count_b: f32,
    pub quantization: String,
    pub total_layers: u32,
    pub total_size_bytes: u64,
    pub piece_size_bytes: u64,
    pub min_vram_gb: f32,
    pub merkle_root_hex: String,
    pub pieces: Vec<ModelPiece>,
}

impl AiTorrentManifest {
    /// Create a standard compute torrent manifest for an AI model
    pub fn new_standard(
        model_id: &str,
        display_name: &str,
        parameter_count_b: f32,
        quantization: &str,
        total_layers: u32,
    ) -> Self {
        // Compute size based on quantization: 4-bit = 0.55 GB/B-params, 8-bit = 1.05 GB/B-params
        let bytes_per_param = match quantization {
            "q4_k_m" | "q4_0" => 0.55,
            "q8_0" | "fp8" => 1.05,
            _ => 2.0,
        };
        let total_size_bytes = (parameter_count_b * bytes_per_param * 1024.0 * 1024.0 * 1024.0) as u64;
        let piece_size_bytes = 16 * 1024 * 1024; // 16 MB piece size (BitTorrent standard)
        let total_pieces = ((total_size_bytes + piece_size_bytes - 1) / piece_size_bytes).max(1);

        let layers_per_piece = ((total_layers as f32) / (total_pieces as f32)).ceil() as u32;

        let mut pieces = Vec::with_capacity(total_pieces as usize);
        let mut piece_hashes = Vec::with_capacity(total_pieces as usize);

        for i in 0..total_pieces {
            let layer_start = (i as u32 * layers_per_piece).min(total_layers);
            let layer_end = ((i as u32 + 1) * layers_per_piece).min(total_layers);
            
            // Deterministic synthetic hash for manifest definition
            let fake_content = format!("{}:{}:{}", model_id, i, total_layers);
            let hash = Blake3Hasher::hash_bytes(fake_content.as_bytes());
            piece_hashes.push(hash);

            pieces.push(ModelPiece {
                piece_index: i as u32,
                layer_start,
                layer_end,
                size_bytes: piece_size_bytes,
                blake3_hash: hex::encode(hash),
            });
        }

        let merkle_root = Blake3Hasher::merkle_root(&piece_hashes);

        Self {
            model_id: model_id.to_string(),
            display_name: display_name.to_string(),
            parameter_count_b,
            quantization: quantization.to_string(),
            total_layers,
            total_size_bytes,
            piece_size_bytes,
            min_vram_gb: (total_size_bytes as f32 / (1024.0 * 1024.0 * 1024.0)) * 0.25, // Min 25% shard
            merkle_root_hex: hex::encode(merkle_root),
            pieces,
        }
    }

    pub fn llama_3_2_3b() -> Self {
        Self::new_standard(
            "llama-3.2-3b-instruct",
            "Llama 3.2 3B Instruct (Q4_K_M)",
            3.21,
            "q4_k_m",
            28,
        )
    }

    pub fn deepseek_r1_1_5b() -> Self {
        Self::new_standard(
            "deepseek-r1-distill-1.5b",
            "DeepSeek R1 Distill Qwen 1.5B (Q4_K_M)",
            1.54,
            "q4_k_m",
            28,
        )
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_creation() {
        let manifest = AiTorrentManifest::new_standard(
            "deepseek-r1-distill-q4",
            "DeepSeek R1 Distill 8B (Q4_K_M)",
            8.0,
            "q4_k_m",
            32,
        );
        assert!(!manifest.pieces.is_empty());
        assert!(!manifest.merkle_root_hex.is_empty());
        assert_eq!(manifest.total_layers, 32);
    }
}
