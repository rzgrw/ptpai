use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey, Signature};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone)]
pub struct NodeIdentity {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
    peer_id: String,
}

impl NodeIdentity {
    /// Generate a new ephemeral or persistent Ed25519 identity
    pub fn generate() -> Self {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();
        let peer_id = hex::encode(verifying_key.as_bytes());

        Self {
            signing_key,
            verifying_key,
            peer_id,
        }
    }

    /// Load or create identity from a 32-byte raw seed
    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(bytes);
        let verifying_key = signing_key.verifying_key();
        let peer_id = hex::encode(verifying_key.as_bytes());

        Self {
            signing_key,
            verifying_key,
            peer_id,
        }
    }

    pub fn peer_id(&self) -> &str {
        &self.peer_id
    }

    pub fn verifying_key(&self) -> &VerifyingKey {
        &self.verifying_key
    }

    pub fn sign(&self, message: &[u8]) -> String {
        let sig: Signature = self.signing_key.sign(message);
        hex::encode(sig.to_bytes())
    }

    pub fn verify_peer(peer_id_hex: &str, message: &[u8], signature_hex: &str) -> bool {
        let key_bytes = match hex::decode(peer_id_hex) {
            Ok(b) => b,
            Err(_) => return false,
        };
        if key_bytes.len() != 32 {
            return false;
        }
        let key_arr: [u8; 32] = key_bytes.as_slice().try_into().unwrap();
        let vk = match VerifyingKey::from_bytes(&key_arr) {
            Ok(k) => k,
            Err(_) => return false,
        };

        let sig_bytes = match hex::decode(signature_hex) {
            Ok(b) => b,
            Err(_) => return false,
        };
        if sig_bytes.len() != 64 {
            return false;
        }
        let sig_arr: [u8; 64] = sig_bytes.as_slice().try_into().unwrap();
        let sig = Signature::from_bytes(&sig_arr);

        vk.verify(message, &sig).is_ok()
    }
}

/// Cryptographically signed compute receipt
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComputeReceipt {
    pub task_id: String,
    pub provider_peer_id: String,
    pub consumer_peer_id: String,
    pub model_id: String,
    pub tokens_processed: u64,
    pub estimated_tflops: f64,
    pub latency_ms: f64,
    pub timestamp_epoch: i64,
    pub signature_hex: String,
}

impl ComputeReceipt {
    pub fn payload_digest(&self) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(self.task_id.as_bytes());
        hasher.update(self.provider_peer_id.as_bytes());
        hasher.update(self.consumer_peer_id.as_bytes());
        hasher.update(self.model_id.as_bytes());
        hasher.update(&self.tokens_processed.to_be_bytes());
        hasher.update(&self.estimated_tflops.to_be_bytes());
        hasher.update(&self.latency_ms.to_be_bytes());
        hasher.update(&self.timestamp_epoch.to_be_bytes());
        *hasher.finalize().as_bytes()
    }

    pub fn new_signed(
        identity: &NodeIdentity,
        task_id: String,
        consumer_peer_id: String,
        model_id: String,
        tokens_processed: u64,
        estimated_tflops: f64,
        latency_ms: f64,
    ) -> Self {
        let timestamp_epoch = chrono::Utc::now().timestamp();
        let mut receipt = Self {
            task_id,
            provider_peer_id: identity.peer_id().to_string(),
            consumer_peer_id,
            model_id,
            tokens_processed,
            estimated_tflops,
            latency_ms,
            timestamp_epoch,
            signature_hex: String::new(),
        };
        let digest = receipt.payload_digest();
        receipt.signature_hex = identity.sign(&digest);
        receipt
    }

    pub fn verify(&self) -> bool {
        let digest = self.payload_digest();
        NodeIdentity::verify_peer(&self.provider_peer_id, &digest, &self.signature_hex)
    }
}

pub struct Blake3Hasher;

impl Blake3Hasher {
    pub fn hash_bytes(data: &[u8]) -> [u8; 32] {
        *blake3::hash(data).as_bytes()
    }

    pub fn hash_file(path: &Path) -> std::io::Result<[u8; 32]> {
        let mut file = std::fs::File::open(path)?;
        let mut hasher = blake3::Hasher::new();
        std::io::copy(&mut file, &mut hasher)?;
        Ok(*hasher.finalize().as_bytes())
    }

    pub fn merkle_root(leaves: &[[u8; 32]]) -> [u8; 32] {
        if leaves.is_empty() {
            return [0u8; 32];
        }
        if leaves.len() == 1 {
            return leaves[0];
        }
        let mut current_level = leaves.to_vec();
        while current_level.len() > 1 {
            let mut next_level = Vec::with_capacity((current_level.len() + 1) / 2);
            for chunk in current_level.chunks(2) {
                let mut hasher = blake3::Hasher::new();
                hasher.update(&chunk[0]);
                if chunk.len() > 1 {
                    hasher.update(&chunk[1]);
                } else {
                    hasher.update(&chunk[0]); // duplicate odd node
                }
                next_level.push(*hasher.finalize().as_bytes());
            }
            current_level = next_level;
        }
        current_level[0]
    }
}

// Simple hex helper module to avoid extra external crate dependencies
pub mod hex {
    pub fn encode<T: AsRef<[u8]>>(data: T) -> String {
        data.as_ref().iter().map(|b| format!("{:02x}", b)).collect()
    }

    pub fn decode(hex_str: &str) -> Result<Vec<u8>, String> {
        if hex_str.len() % 2 != 0 {
            return Err("Invalid hex string length".to_string());
        }
        (0..hex_str.len())
            .step_by(2)
            .map(|i| {
                u8::from_str_radix(&hex_str[i..i + 2], 16)
                    .map_err(|e| format!("Failed to parse hex at {}: {}", i, e))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_signature_verification() {
        let id = NodeIdentity::generate();
        let message = b"Compute proof verification for shard 12";
        let sig = id.sign(message);
        assert!(NodeIdentity::verify_peer(id.peer_id(), message, &sig));
        assert!(!NodeIdentity::verify_peer(id.peer_id(), b"tampered", &sig));
    }

    #[test]
    fn test_compute_receipt_signing() {
        let provider = NodeIdentity::generate();
        let receipt = ComputeReceipt::new_signed(
            &provider,
            "task-987".to_string(),
            "consumer-456".to_string(),
            "deepseek-r1-q4".to_string(),
            128,
            2.4,
            45.2,
        );
        assert!(receipt.verify());
    }

    #[test]
    fn test_merkle_root() {
        let leaf1 = Blake3Hasher::hash_bytes(b"piece 0");
        let leaf2 = Blake3Hasher::hash_bytes(b"piece 1");
        let root = Blake3Hasher::merkle_root(&[leaf1, leaf2]);
        assert_ne!(root, [0u8; 32]);
    }
}
