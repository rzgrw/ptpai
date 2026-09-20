use bytes::{Buf, BufMut, Bytes, BytesMut};
use crate::crypto::aead::P2pAeadCipher;
use std::io;

pub const MAGIC_BYTES: &[u8; 4] = b"AIT1";
pub const HEADER_SIZE: usize = 32;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    Handshake = 0x01,
    Heartbeat = 0x02,
    PeerAnnounce = 0x03,
    TaskSubmit = 0x10,
    TaskAccept = 0x11,
    ActivationChunk = 0x12,
    TokenStream = 0x13,
    TaskComplete = 0x14,
    ComputeReceipt = 0x20,
    CanaryChallenge = 0x30,
    CanaryResponse = 0x31,
    Choke = 0x40,
    Unchoke = 0x41,
    Error = 0xFF,
}

impl MessageType {
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            0x01 => Some(Self::Handshake),
            0x02 => Some(Self::Heartbeat),
            0x03 => Some(Self::PeerAnnounce),
            0x10 => Some(Self::TaskSubmit),
            0x11 => Some(Self::TaskAccept),
            0x12 => Some(Self::ActivationChunk),
            0x13 => Some(Self::TokenStream),
            0x14 => Some(Self::TaskComplete),
            0x20 => Some(Self::ComputeReceipt),
            0x30 => Some(Self::CanaryChallenge),
            0x31 => Some(Self::CanaryResponse),
            0x40 => Some(Self::Choke),
            0x41 => Some(Self::Unchoke),
            0xFF => Some(Self::Error),
            _ => None,
        }
    }
}

/// Zero-copy 32-byte wire header
/// Layout:
/// [0..4]   Magic b"AIT1"
/// [4]      Message Type (u8)
/// [5]      Flags (bit 0: is_canary, bit 1: compressed, bit 2: requires_receipt)
/// [6..8]   Reserved (2 bytes)
/// [8..16]  Sequence ID (u64 big-endian)
/// [16..20] Payload length (u32 big-endian)
/// [20..32] Truncated BLAKE3 checksum (12 bytes)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketHeader {
    pub msg_type: MessageType,
    pub flags: u8,
    pub sequence_id: u64,
    pub payload_len: u32,
    pub blake3_checksum: [u8; 12],
}

impl PacketHeader {
    pub const FLAG_CANARY: u8 = 1 << 0;
    pub const FLAG_COMPRESSED: u8 = 1 << 1;
    pub const FLAG_RECEIPT_REQ: u8 = 1 << 2;
    pub const FLAG_ENCRYPTED: u8 = 1 << 3;

    pub fn new(msg_type: MessageType, sequence_id: u64, payload: &[u8]) -> Self {
        let hash = blake3::hash(payload);
        let mut checksum = [0u8; 12];
        checksum.copy_from_slice(&hash.as_bytes()[..12]);

        Self {
            msg_type,
            flags: 0,
            sequence_id,
            payload_len: payload.len() as u32,
            blake3_checksum: checksum,
        }
    }

    pub fn with_flags(mut self, flags: u8) -> Self {
        self.flags = flags;
        self
    }

    pub fn is_canary(&self) -> bool {
        (self.flags & Self::FLAG_CANARY) != 0
    }

    pub fn is_compressed(&self) -> bool {
        (self.flags & Self::FLAG_COMPRESSED) != 0
    }

    pub fn is_encrypted(&self) -> bool {
        (self.flags & Self::FLAG_ENCRYPTED) != 0
    }

    /// Fixed 14-byte Associated Authenticated Data (AAD) for AEAD cipher: [Magic (4) | MsgType (1) | Flags (1) | SeqId (8)]
    pub fn aad_bytes(&self) -> [u8; 14] {
        let mut aad = [0u8; 14];
        aad[..4].copy_from_slice(MAGIC_BYTES);
        aad[4] = self.msg_type as u8;
        aad[5] = self.flags | Self::FLAG_ENCRYPTED;
        aad[6..14].copy_from_slice(&self.sequence_id.to_be_bytes());
        aad
    }

    pub fn encode(&self, dst: &mut BytesMut) {
        dst.put_slice(MAGIC_BYTES);
        dst.put_u8(self.msg_type as u8);
        dst.put_u8(self.flags);
        dst.put_u16(0); // reserved
        dst.put_u64(self.sequence_id);
        dst.put_u32(self.payload_len);
        dst.put_slice(&self.blake3_checksum);
    }

    pub fn decode(src: &mut Bytes) -> io::Result<Self> {
        if src.len() < HEADER_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Incomplete packet header",
            ));
        }

        let magic = &src[..4];
        if magic != MAGIC_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid magic bytes: {:?}", magic),
            ));
        }

        let msg_type_byte = src[4];
        let msg_type = MessageType::from_u8(msg_type_byte).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unknown message type: 0x{:02X}", msg_type_byte),
            )
        })?;

        let flags = src[5];
        // reserved 2 bytes: src[6..8]
        let sequence_id = u64::from_be_bytes(src[8..16].try_into().unwrap());
        let payload_len = u32::from_be_bytes(src[16..20].try_into().unwrap());
        let mut blake3_checksum = [0u8; 12];
        blake3_checksum.copy_from_slice(&src[20..32]);

        src.advance(HEADER_SIZE);

        Ok(Self {
            msg_type,
            flags,
            sequence_id,
            payload_len,
            blake3_checksum,
        })
    }

    pub fn verify_payload(&self, payload: &[u8]) -> bool {
        if payload.len() != self.payload_len as usize {
            return false;
        }
        let hash = blake3::hash(payload);
        &hash.as_bytes()[..12] == self.blake3_checksum
    }
}

/// High performance wire frame containing header and zero-copy payload bytes
#[derive(Debug, Clone)]
pub struct WireFrame {
    pub header: PacketHeader,
    pub payload: Bytes,
}

impl WireFrame {
    pub fn new(msg_type: MessageType, sequence_id: u64, payload: Bytes) -> Self {
        let header = PacketHeader::new(msg_type, sequence_id, &payload);
        Self { header, payload }
    }

    pub fn encode(&self) -> Bytes {
        let mut buf = BytesMut::with_capacity(HEADER_SIZE + self.payload.len());
        self.header.encode(&mut buf);
        buf.put_slice(&self.payload);
        buf.freeze()
    }

    pub fn decode(mut src: Bytes) -> io::Result<Self> {
        let header = PacketHeader::decode(&mut src)?;
        if src.len() < header.payload_len as usize {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Payload size is smaller than header declaration",
            ));
        }
        let payload = src.split_to(header.payload_len as usize);
        if !header.verify_payload(&payload) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "BLAKE3 payload checksum mismatch",
            ));
        }
        Ok(Self { header, payload })
    }

    /// Encrypt frame payload in-place using ChaCha20-Poly1305 AEAD with header as authenticated data
    pub fn encrypt_with_cipher(&mut self, cipher: &P2pAeadCipher) -> Result<(), String> {
        let aad = self.header.aad_bytes();
        let (ciphertext, _) = cipher.encrypt(self.header.sequence_id, &self.payload, &aad)?;
        let cipher_hash = blake3::hash(&ciphertext);

        self.header.flags |= PacketHeader::FLAG_ENCRYPTED;
        self.header.payload_len = ciphertext.len() as u32;
        self.header.blake3_checksum.copy_from_slice(&cipher_hash.as_bytes()[..12]);
        self.payload = Bytes::from(ciphertext);

        Ok(())
    }

    /// Decrypt and verify Poly1305 MAC of payload in-place
    pub fn decrypt_with_cipher(&mut self, cipher: &P2pAeadCipher) -> Result<(), String> {
        if !self.header.is_encrypted() {
            return Ok(());
        }

        let aad = self.header.aad_bytes();
        let plaintext = cipher.decrypt(self.header.sequence_id, &self.payload, &aad)?;
        let plain_hash = blake3::hash(&plaintext);

        self.header.flags &= !PacketHeader::FLAG_ENCRYPTED;
        self.header.payload_len = plaintext.len() as u32;
        self.header.blake3_checksum.copy_from_slice(&plain_hash.as_bytes()[..12]);
        self.payload = Bytes::from(plaintext);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wire_frame_roundtrip() {
        let payload = Bytes::from_static(b"Hello Apple Silicon Unified Memory Swarm");
        let frame = WireFrame::new(MessageType::TokenStream, 42, payload.clone());

        let encoded = frame.encode();
        assert_eq!(encoded.len(), HEADER_SIZE + payload.len());

        let decoded = WireFrame::decode(encoded).expect("decode failed");
        assert_eq!(decoded.header.sequence_id, 42);
        assert_eq!(decoded.header.msg_type, MessageType::TokenStream);
        assert_eq!(decoded.payload, payload);
    }

    #[test]
    fn test_tampered_payload_detection() {
        let payload = Bytes::from_static(b"Original honest computation");
        let frame = WireFrame::new(MessageType::ActivationChunk, 1, payload);
        let mut encoded = frame.encode().to_vec();

        // Tamper with one byte in the payload
        let last_idx = encoded.len() - 1;
        encoded[last_idx] ^= 0xFF;

        let res = WireFrame::decode(Bytes::from(encoded));
        assert!(res.is_err(), "Must detect tampered payload via BLAKE3");
    }

    #[test]
    fn test_aead_wire_frame_encryption() {
        let key = [0x42u8; 32];
        let cipher = P2pAeadCipher::new(&key);

        let original_payload = Bytes::from_static(b"Secret Activation Tensor: [1.23, 4.56, -7.89]");
        let mut frame = WireFrame::new(MessageType::ActivationChunk, 101, original_payload.clone());

        // Encrypt in-place
        frame.encrypt_with_cipher(&cipher).expect("encryption failed");
        assert!(frame.header.is_encrypted());
        assert_ne!(frame.payload, original_payload);

        // Encode to wire format and decode
        let wire_bytes = frame.encode();
        let mut decoded_frame = WireFrame::decode(wire_bytes).expect("wire decode failed");
        assert!(decoded_frame.header.is_encrypted());

        // Decrypt in-place
        decoded_frame.decrypt_with_cipher(&cipher).expect("decryption failed");
        assert!(!decoded_frame.header.is_encrypted());
        assert_eq!(decoded_frame.payload, original_payload);
    }
}
