use crate::crypto::{Blake3Hasher, hex};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Compute Tit-for-Tat (cT4T) ledger tracking compute balance per peer
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

    /// Compute fair-share ratio with optimistic starter credits (100 tokens free)
    pub fn ratio(&self) -> f64 {
        let served = self.tokens_served as f64 + 100.0;
        let consumed = self.tokens_consumed as f64 + 100.0;
        served / consumed
    }

    pub fn update_choke_status(&mut self) {
        // Choke if ratio drops below 0.25 and consumed significant tokens (> 500)
        if self.tokens_consumed > 500 && self.ratio() < 0.25 {
            self.is_choked = true;
        } else if self.failed_or_cheated_jobs > 0 {
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
    /// Local trust matrix: C[i][j] = trust node i places in node j
    local_trust: HashMap<String, HashMap<String, f64>>,
    /// Global trust scores computed via power iteration
    global_trust: HashMap<String, f64>,
    /// Permanently slashed fraudulent peers
    pub slashed_peers: std::collections::HashSet<String>,
    /// Audit log of detected cheats
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

    /// Record a local transaction outcome between rater and ratee
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

    /// Record a canary fraud catch
    pub fn record_fraud(&mut self, peer_id: &str, reason: &str, details: &str) {
        self.audit_logs.push(SecurityAuditLog {
            incident_id: format!("audit-{}", self.audit_logs.len() + 1),
            peer_id: peer_id.to_string(),
            incident_type: reason.to_string(),
            details: details.to_string(),
            timestamp_epoch: chrono::Utc::now().timestamp(),
        });

        // Slash all trust ratings for this malicious peer
        self.slashed_peers.insert(peer_id.to_string());
        for rater_map in self.local_trust.values_mut() {
            if let Some(rating) = rater_map.get_mut(peer_id) {
                *rating = 0.0;
            }
        }
        self.global_trust.insert(peer_id.to_string(), 0.0);
    }

    /// Compute normalized global trust vector via EigenTrust power iteration
    /// t(k+1) = (1 - a) * C^T * t(k) + a * p
    pub fn compute_eigentrust(&mut self, pre_trusted_peers: &[String]) {
        if self.local_trust.is_empty() && pre_trusted_peers.is_empty() {
            return;
        }

        // Collect all unique peer IDs
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

        // Build normalized matrix C: c_ij = max(s_ij, 0) / sum(max(s_ik, 0))
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

        // Pre-trusted distribution vector p
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

        // Power iteration
        let alpha = 0.15; // standard probability of jumping to pre-trusted seed
        let mut t = p.clone();

        for _ in 0..25 {
            let mut t_next = vec![0.0f64; n];

            // t_next = (1 - alpha) * C^T * t + alpha * p
            for j in 0..n {
                let mut sum_c_t = 0.0;
                for i in 0..n {
                    sum_c_t += c_matrix[i][j] * t[i];
                }
                t_next[j] = (1.0 - alpha) * sum_c_t + alpha * p[j];
            }

            // Check L1 convergence
            let diff: f64 = t.iter().zip(&t_next).map(|(a, b)| (a - b).abs()).sum();
            t = t_next;
            if diff < 1e-4 {
                break;
            }
        }

        // Save scores back to global map
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

/// Stealth Canary challenge manager
pub struct CanaryEngine {
    known_canaries: HashMap<String, String>, // prompt -> expected_hash
}

impl CanaryEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            known_canaries: HashMap::new(),
        };
        // Register deterministic canary test vectors
        engine.register("CANARY_TEST_ALPHA_98", "68e144a62dc4");
        engine.register("CANARY_TEST_BETA_42", "9a7bc01ef832");
        engine.register("CANARY_TEST_GAMMA_11", "0bf431cd65e9");
        engine
    }

    pub fn register(&mut self, prompt: &str, expected_hash_prefix: &str) {
        self.known_canaries
            .insert(prompt.to_string(), expected_hash_prefix.to_string());
    }

    /// Craft a canary prompt to inject into a worker stream
    pub fn sample_canary(&self) -> Option<(&str, &str)> {
        self.known_canaries
            .iter()
            .next()
            .map(|(k, v)| (k.as_str(), v.as_str()))
    }

    /// Verify if returned activation hash matches ground truth
    pub fn verify_canary(&self, prompt: &str, returned_hash: &str) -> bool {
        if let Some(expected) = self.known_canaries.get(prompt) {
            returned_hash.starts_with(expected)
        } else {
            // If unknown canary, verify via BLAKE3 deterministic hash
            let computed = hex::encode(Blake3Hasher::hash_bytes(prompt.as_bytes()));
            returned_hash.starts_with(&computed[..12])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eigentrust_convergence() {
        let mut engine = EigenTrustEngine::new();
        let peer_a = "peer-alice".to_string();
        let peer_b = "peer-bob".to_string();
        let peer_c = "peer-charlie".to_string();

        engine.record_transaction(&peer_a, &peer_b, true, 100);
        engine.record_transaction(&peer_b, &peer_c, true, 100);
        engine.record_transaction(&peer_c, &peer_b, true, 50);

        engine.compute_eigentrust(&[peer_a.clone()]);
        let bob_score = engine.get_score(&peer_b);
        assert!(bob_score > 0.0);
    }

    #[test]
    fn test_fraud_slashing() {
        let mut engine = EigenTrustEngine::new();
        let honest = "peer-honest".to_string();
        let malicious = "peer-malicious".to_string();

        engine.record_transaction(&honest, &malicious, true, 50);
        engine.record_fraud(&malicious, "CANARY_MISMATCH", "Returned 0x00 instead of logits");

        let score = engine.get_score(&malicious);
        assert_eq!(score, 0.0);
        assert_eq!(engine.audit_logs.len(), 1);
    }
}
