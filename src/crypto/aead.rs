use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    ChaCha20Poly1305, Key, Nonce,
};
use rand::rngs::OsRng;
use x25519_dalek::{EphemeralSecret, PublicKey};

pub const KDF_CONTEXT: &str = "PTPAI P2P AEAD SESSION KEY V1";

/// Ephemeral X25519 session keypair for mutual P2P Diffie-Hellman negotiation
pub struct SessionKeyExchange {
    secret: Option<EphemeralSecret>,
    pub public_key: PublicKey,
}

impl SessionKeyExchange {
    pub fn new() -> Self {
        let mut rng = OsRng;
        let secret = EphemeralSecret::random_from_rng(&mut rng);
        let public_key = PublicKey::from(&secret);

        Self {
            secret: Some(secret),
            public_key,
        }
    }

    pub fn public_key_bytes(&self) -> [u8; 32] {
        *self.public_key.as_bytes()
    }

    /// Complete Diffie-Hellman with peer's public key to derive ChaCha20-Poly1305 cipher
    pub fn derive_session_cipher(
        mut self,
        peer_public_key_bytes: &[u8; 32],
    ) -> Result<P2pAeadCipher, String> {
        let secret = self
            .secret
            .take()
            .ok_or_else(|| "Ephemeral secret already consumed".to_string())?;

        let peer_pk = PublicKey::from(*peer_public_key_bytes);
        let shared_secret = secret.diffie_hellman(&peer_pk);

        // Derive 256-bit symmetric session key using BLAKE3 KDF
        let mut hasher = blake3::Hasher::new_derive_key(KDF_CONTEXT);
        hasher.update(shared_secret.as_bytes());
        let session_key = *hasher.finalize().as_bytes();

        Ok(P2pAeadCipher::new(&session_key))
    }
}

/// ChaCha20-Poly1305 Authenticated Encryption with Associated Data (AEAD)
#[derive(Clone)]
pub struct P2pAeadCipher {
    cipher: ChaCha20Poly1305,
    key_bytes: [u8; 32],
}

impl P2pAeadCipher {
    pub fn new(key_bytes: &[u8; 32]) -> Self {
        let key = Key::from_slice(key_bytes);
        let cipher = ChaCha20Poly1305::new(key);
        Self {
            cipher,
            key_bytes: *key_bytes,
        }
    }

    pub fn key_bytes(&self) -> &[u8; 32] {
        &self.key_bytes
    }

    /// Construct 12-byte AEAD nonce from monotonic sequence ID
    pub fn construct_nonce(sequence_id: u64, direction: u32) -> [u8; 12] {
        let mut nonce = [0u8; 12];
        nonce[..8].copy_from_slice(&sequence_id.to_be_bytes());
        nonce[8..12].copy_from_slice(&direction.to_be_bytes());
        nonce
    }

    /// Encrypt payload with Associated Authenticated Data (AAD)
    pub fn encrypt(
        &self,
        sequence_id: u64,
        plaintext: &[u8],
        associated_data: &[u8],
    ) -> Result<(Vec<u8>, [u8; 12]), String> {
        let nonce_bytes = Self::construct_nonce(sequence_id, 0x01);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let payload = Payload {
            msg: plaintext,
            aad: associated_data,
        };

        let ciphertext = self
            .cipher
            .encrypt(nonce, payload)
            .map_err(|e| format!("AEAD encryption failed: {}", e))?;

        Ok((ciphertext, nonce_bytes))
    }

    /// Decrypt and authenticate ciphertext with Associated Authenticated Data (AAD)
    pub fn decrypt(
        &self,
        sequence_id: u64,
        ciphertext: &[u8],
        associated_data: &[u8],
    ) -> Result<Vec<u8>, String> {
        let nonce_bytes = Self::construct_nonce(sequence_id, 0x01);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let payload = Payload {
            msg: ciphertext,
            aad: associated_data,
        };

        let plaintext = self
            .cipher
            .decrypt(nonce, payload)
            .map_err(|e| format!("AEAD authentication or decryption failed: {}", e))?;

        Ok(plaintext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diffie_hellman_session_derivation() {
        let alice = SessionKeyExchange::new();
        let bob = SessionKeyExchange::new();

        let alice_pk = alice.public_key_bytes();
        let bob_pk = bob.public_key_bytes();

        let alice_cipher = alice.derive_session_cipher(&bob_pk).unwrap();
        let bob_cipher = bob.derive_session_cipher(&alice_pk).unwrap();

        // Symmetrical session keys must match exactly
        assert_eq!(alice_cipher.key_bytes(), bob_cipher.key_bytes());
    }

    #[test]
    fn test_aead_encryption_roundtrip() {
        let alice = SessionKeyExchange::new();
        let bob = SessionKeyExchange::new();

        let alice_pk = alice.public_key_bytes();
        let bob_pk = bob.public_key_bytes();

        let alice_cipher = alice.derive_session_cipher(&bob_pk).unwrap();
        let bob_cipher = bob.derive_session_cipher(&alice_pk).unwrap();

        let plaintext = b"Confidential Activation Tensor: [0.341, -0.912, 1.455]";
        let aad = b"AIT1\x13\x08seq_42_header";

        // Alice encrypts
        let (ciphertext, _) = alice_cipher.encrypt(42, plaintext, aad).unwrap();
        assert_ne!(&ciphertext, plaintext);

        // Bob decrypts and verifies MAC
        let decrypted = bob_cipher.decrypt(42, &ciphertext, aad).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_aead_tampering_rejection() {
        let alice = SessionKeyExchange::new();
        let bob = SessionKeyExchange::new();

        let alice_pk = alice.public_key_bytes();
        let bob_pk = bob.public_key_bytes();

        let alice_cipher = alice.derive_session_cipher(&bob_pk).unwrap();
        let bob_cipher = bob.derive_session_cipher(&alice_pk).unwrap();

        let plaintext = b"Private prompt text";
        let aad = b"header_data";

        let (mut ciphertext, _) = alice_cipher.encrypt(1, plaintext, aad).unwrap();

        // Tamper with 1 byte of ciphertext (simulating an adversary in transit)
        let last = ciphertext.len() - 1;
        ciphertext[last] ^= 0x01;

        let res = bob_cipher.decrypt(1, &ciphertext, aad);
        assert!(res.is_err(), "Must reject tampered ciphertext with Poly1305 MAC failure");
    }
}
