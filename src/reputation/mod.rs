use crate::crypto::{Blake3Hasher, hex};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EconomicGateError {
    pub reason: String,
    pub current_ratio: f64,
    pub required_ratio: f64,
    pub tokens_served: u64,
    pub required_tokens: u64,
    pub unlocked: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModelEconomicRequirements {
    pub min_ratio: f64,
    pub min_tokens_served: u64,
    pub min_eigentrust: f64,
}

impl ModelEconomicRequirements {
    pub fn for_model(model_id: &str) -> Self {
        match model_id {
            // Tier 1: Starter lightweight model (Low barrier to entry, quick unlock)
            "deepseek-r1-distill-1.5b" => Self {
                min_ratio: 0.80,
                min_tokens_served: 64,
                min_eigentrust: 0.40,
            },
            // Tier 2: Standard production model (Requires 1.00 fair-share ratio)
            "llama-3.2-3b-instruct" => Self {
                min_ratio: 1.00,
                min_tokens_served: 128,
                min_eigentrust: 0.50,
            },
            // Tier 3: Flagship heavy reasoning model (Strict high contribution requirement)
            "deepseek-r1-distill-8b" => Self {
                min_ratio: 1.50,
                min_tokens_served: 500,
                min_eigentrust: 0.70,
            },
            _ => Self {
                min_ratio: 1.00,
                min_tokens_served: 128,
                min_eigentrust: 0.50,
            },
        }
    }
}

/// Compute Tit-for-Tat (cT4T) ledger tracking strict compute balance per peer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerRatioRecord {
    pub peer_id: String,
    pub tokens_served: u64,
    pub tokens_consumed: u64,
    pub tflops_served: f64,
    pub tflops_consumed: f64,
    pub successful_jobs: u64,
    pub failed_or_cheated_jobs: u64,
    pub last_seen_epoch: i64,
    pub is_choked: bool,
}

impl PeerRatioRecord {
    pub fn new(peer_id: String) -> Self {
        Self {
            peer_id,
            tokens_served: 0,
            tokens_consumed: 0,
            tflops_served: 0.0,
            tflops_consumed: 0.0,
            successful_jobs: 0,
            failed_or_cheated_jobs: 0,
            last_seen_epoch: chrono::Utc::now().timestamp(),
            is_choked: false,
        }
    }

    /// Strict economic ratio: tokens contributed / tokens consumed.
    /// Pure seeders with 0 consumed get high ratio based on served tokens.
    pub fn ratio(&self) -> f64 {
        if self.tokens_consumed == 0 {
            if self.tokens_served == 0 {
                0.0
            } else {
                (self.tokens_served as f64).min(99.0) // high ratio for pure contributors
            }
        } else {
            (self.tokens_served as f64) / (self.tokens_consumed as f64)
        }
    }

    /// Check if peer has earned enough ratio and served enough tokens to use this model
    pub fn verify_economic_access(
        &self,
        model_id: &str,
        eigentrust_score: f64,
    ) -> Result<(), EconomicGateError> {
        if self.is_choked || self.failed_or_cheated_jobs > 0 {
            return Err(EconomicGateError {
                reason: "Node is CHOKED by swarm due to failed verification or excessive debt".to_string(),
                current_ratio: self.ratio(),
                required_ratio: 1.0,
                tokens_served: self.tokens_served,
                required_tokens: 128,
                unlocked: false,
            });
        }

        let req = ModelEconomicRequirements::for_model(model_id);

        // Bootstrap check: Must have seeded minimum tokens first
        if self.tokens_served < req.min_tokens_served {
            return Err(EconomicGateError {
                reason: format!(
                    "Proof-of-Seeding Required: You have served {}/{} tokens for this model tier. Please leave your browser tab or node seeding compute to earn access.",
                    self.tokens_served, req.min_tokens_served
                ),
                current_ratio: self.ratio(),
                required_ratio: req.min_ratio,
                tokens_served: self.tokens_served,
                required_tokens: req.min_tokens_served,
                unlocked: false,
            });
        }

        // Ratio requirement check
        let cur_ratio = self.ratio();
        if cur_ratio < req.min_ratio {
            return Err(EconomicGateError {
                reason: format!(
                    "Ratio Too Low: Your compute ratio is {:.2}, but '{}' requires a minimum ratio of >= {:.2}. Seed compute to raise your standing.",
                    cur_ratio, model_id, req.min_ratio
                ),
                current_ratio: cur_ratio,
                required_ratio: req.min_ratio,
                tokens_served: self.tokens_served,
                required_tokens: req.min_tokens_served,
                unlocked: false,
            });
        }

        // EigenTrust reputation threshold check
        if eigentrust_score < req.min_eigentrust {
            return Err(EconomicGateError {
                reason: format!(
                    "Insufficient EigenTrust Score: Your trust score is {:.2}, but this tier requires >= {:.2}.",
                    eigentrust_score, req.min_eigentrust
                ),
                current_ratio: cur_ratio,
                required_ratio: req.min_ratio,
                tokens_served: self.tokens_served,
                required_tokens: req.min_tokens_served,
                unlocked: false,
            });
        }

        Ok(())
    }

    pub fn update_choke_status(&mut self) {
        if self.failed_or_cheated_jobs > 0 {
            self.is_choked = true;
        } else if self.tokens_consumed > 100 && self.ratio() < 0.75 {
            self.is_choked = true;
        } else {
            self.is_choked = false;
        }
    }
}

/// Anti-fraud audit record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAuditLog {
    pub incident_id: String,
    pub peer_id: String,
    pub incident_type: String, // "CANARY_MISMATCH", "INVALID_SIGNATURE", "CHECKSUM_FAIL"
    pub details: String,
    pub timestamp_epoch: i64,
}

/// EigenTrust implementation for decentralized global peer trust aggregation
pub struct EigenTrustEngine {
    local_trust: HashMap<String, HashMap<String, f64>>,
    global_trust: HashMap<String, f64>,
    pub slashed_peers: std::collections::HashSet<String>,
    pub audit_logs: Vec<SecurityAuditLog>,
}

impl EigenTrustEngine {
    pub fn new() -> Self {
        Self {
            local_trust: HashMap::new(),
            global_trust: HashMap::new(),
            slashed_peers: std::collections::HashSet::new(),
            audit_logs: Vec::new(),
        }
    }

    pub fn record_transaction(
        &mut self,
        rater_peer_id: &str,
        ratee_peer_id: &str,
        is_satisfactory: bool,
        tokens: u64,
    ) {
        let rater_map = self
            .local_trust
            .entry(rater_peer_id.to_string())
            .or_insert_with(HashMap::new);

        let weight = (tokens as f64).ln_1p().max(1.0);
        let entry = rater_map.entry(ratee_peer_id.to_string()).or_insert(0.0);

        if is_satisfactory {
            *entry += weight;
        } else {
            *entry = (*entry - weight * 3.0).max(0.0);
        }
    }

    pub fn record_fraud(&mut self, peer_id: &str, reason: &str, details: &str) {
        self.audit_logs.push(SecurityAuditLog {
            incident_id: format!("audit-{}", self.audit_logs.len() + 1),
            peer_id: peer_id.to_string(),
            incident_type: reason.to_string(),
            details: details.to_string(),
            timestamp_epoch: chrono::Utc::now().timestamp(),
        });

        self.slashed_peers.insert(peer_id.to_string());
        for rater_map in self.local_trust.values_mut() {
            if let Some(rating) = rater_map.get_mut(peer_id) {
                *rating = 0.0;
            }
        }
        self.global_trust.insert(peer_id.to_string(), 0.0);
    }

    pub fn compute_eigentrust(&mut self, pre_trusted_peers: &[String]) {
        if self.local_trust.is_empty() && pre_trusted_peers.is_empty() {
            return;
        }

        let mut all_peers: Vec<String> = self.local_trust.keys().cloned().collect();
        for peer in pre_trusted_peers {
            if !all_peers.contains(peer) {
                all_peers.push(peer.clone());
            }
        }
        for sub_map in self.local_trust.values() {
            for peer in sub_map.keys() {
                if !all_peers.contains(peer) {
                    all_peers.push(peer.clone());
                }
            }
        }

        let n = all_peers.len();
        if n == 0 {
            return;
        }

        let peer_index: HashMap<String, usize> = all_peers
            .iter()
            .enumerate()
            .map(|(i, id)| (id.clone(), i))
            .collect();

        let mut c_matrix = vec![vec![0.0f64; n]; n];
        for (rater, ratings) in &self.local_trust {
            if let Some(&i) = peer_index.get(rater) {
                let sum: f64 = ratings.values().map(|v| v.max(0.0)).sum();
                if sum > 0.0 {
                    for (ratee, &val) in ratings {
                        if let Some(&j) = peer_index.get(ratee) {
                            c_matrix[i][j] = val.max(0.0) / sum;
                        }
                    }
                }
            }
        }

        let mut p = vec![0.0f64; n];
        let p_count = pre_trusted_peers.len();
        if p_count > 0 {
            let p_val = 1.0 / (p_count as f64);
            for id in pre_trusted_peers {
                if let Some(&idx) = peer_index.get(id) {
                    p[idx] = p_val;
                }
            }
        } else {
            let p_val = 1.0 / (n as f64);
            for item in p.iter_mut() {
                *item = p_val;
            }
        }

        let alpha = 0.15;
        let mut t = p.clone();

        for _ in 0..25 {
            let mut t_next = vec![0.0f64; n];

            for j in 0..n {
                let mut sum_c_t = 0.0;
                for i in 0..n {
                    sum_c_t += c_matrix[i][j] * t[i];
                }
                t_next[j] = (1.0 - alpha) * sum_c_t + alpha * p[j];
            }

            let diff: f64 = t.iter().zip(&t_next).map(|(a, b)| (a - b).abs()).sum();
            t = t_next;
            if diff < 1e-4 {
                break;
            }
        }

        self.global_trust.clear();
        for (id, &idx) in &peer_index {
            let score = if self.slashed_peers.contains(id) {
                0.0
            } else {
                t[idx]
            };
            self.global_trust.insert(id.clone(), score);
        }
    }

    pub fn get_score(&self, peer_id: &str) -> f64 {
        if self.slashed_peers.contains(peer_id) {
            return 0.0;
        }
        self.global_trust.get(peer_id).copied().unwrap_or(0.5)
    }

    pub fn get_leaderboard(&self) -> Vec<(String, f64)> {
        let mut scores: Vec<(String, f64)> = self
            .global_trust
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores
    }
}

pub struct CanaryEngine {
    known_canaries: HashMap<String, String>,
}

impl CanaryEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            known_canaries: HashMap::new(),
        };
        engine.register("CANARY_TEST_ALPHA_98", "68e144a62dc4");
        engine.register("CANARY_TEST_BETA_42", "9a7bc01ef832");
        engine.register("CANARY_TEST_GAMMA_11", "0bf431cd65e9");
        engine
    }

    pub fn register(&mut self, prompt: &str, expected_hash_prefix: &str) {
        self.known_canaries
            .insert(prompt.to_string(), expected_hash_prefix.to_string());
    }

    pub fn sample_canary(&self) -> Option<(&str, &str)> {
        self.known_canaries
            .iter()
            .next()
            .map(|(k, v)| (k.as_str(), v.as_str()))
    }

    pub fn verify_canary(&self, prompt: &str, returned_hash: &str) -> bool {
        if let Some(expected) = self.known_canaries.get(prompt) {
            returned_hash.starts_with(expected)
        } else {
            let computed = hex::encode(Blake3Hasher::hash_bytes(prompt.as_bytes()));
            returned_hash.starts_with(&computed[..12])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_economic_gating() {
        let mut peer = PeerRatioRecord::new("new-freeloader".to_string());
        
        // Fresh peer has 0 tokens served -> blocked from model inference
        let res = peer.verify_economic_access("llama-3.2-3b-instruct", 0.5);
        assert!(res.is_err(), "Fresh peer with 0 tokens served must be blocked");

        // Peer seeds 150 tokens into the swarm -> unlocks Tier 2 model
        peer.tokens_served = 150;
        let res = peer.verify_economic_access("llama-3.2-3b-instruct", 0.5);
        assert!(res.is_ok(), "Peer with 150 tokens served and 0 consumed has high ratio -> unlocked");

        // Peer consumes 200 tokens (ratio drops to 150/200 = 0.75) -> blocked until they seed more
        peer.tokens_consumed = 200;
        let res = peer.verify_economic_access("llama-3.2-3b-instruct", 0.5);
        assert!(res.is_err(), "Peer with 0.75 ratio must be blocked from Tier 2 (requires 1.00)");
    }
}
