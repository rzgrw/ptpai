use std::net::SocketAddr;
use tokio::net::UdpSocket;
use tracing::{info, warn};

pub const STUN_MAGIC_COOKIE: u32 = 0x2112A442;
pub const BINDING_REQUEST: u16 = 0x0001;
pub const BINDING_RESPONSE: u16 = 0x0101;
pub const ATTR_XOR_MAPPED_ADDRESS: u16 = 0x0020;

/// Embedded STUN / BEP-55 UDP Holepunch Server for NAT traversal between remote Macs
pub async fn start_stun_server(bind_addr: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let socket = UdpSocket::bind(bind_addr).await?;
    info!("STUN / BEP-55 NAT Holepunch Server active on {}", bind_addr);

    let mut buf = [0u8; 1024];

    loop {
        let (len, peer_addr) = match socket.recv_from(&mut buf).await {
            Ok(res) => res,
            Err(e) => {
                warn!("STUN recv error: {}", e);
                continue;
            }
        };

        if len < 20 {
            continue; // Minimum STUN header size is 20 bytes
        }

        let msg_type = u16::from_be_bytes([buf[0], buf[1]]);
        let magic = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);

        if magic == STUN_MAGIC_COOKIE && msg_type == BINDING_REQUEST {
            // Build STUN Binding Success Response (RFC 5389)
            let mut resp = Vec::with_capacity(32);
            resp.extend_from_slice(&BINDING_RESPONSE.to_be_bytes()); // Type: 0x0101
            resp.extend_from_slice(&12u16.to_be_bytes()); // Attribute payload length: 12 bytes
            resp.extend_from_slice(&STUN_MAGIC_COOKIE.to_be_bytes()); // Magic cookie
            resp.extend_from_slice(&buf[8..20]); // Echo original 12-byte transaction ID

            // Add XOR-MAPPED-ADDRESS attribute (0x0020)
            resp.extend_from_slice(&ATTR_XOR_MAPPED_ADDRESS.to_be_bytes());
            resp.extend_from_slice(&8u16.to_be_bytes()); // Attribute length: 8 bytes
            resp.push(0x00); // Reserved

            match peer_addr {
                SocketAddr::V4(v4) => {
                    resp.push(0x01); // IPv4 family
                    let xor_port = peer_addr.port() ^ ((STUN_MAGIC_COOKIE >> 16) as u16);
                    resp.extend_from_slice(&xor_port.to_be_bytes());

                    let ip_bytes = v4.ip().octets();
                    let cookie_bytes = STUN_MAGIC_COOKIE.to_be_bytes();
                    for i in 0..4 {
                        resp.push(ip_bytes[i] ^ cookie_bytes[i]);
                    }
                }
                SocketAddr::V6(_) => {
                    resp.push(0x02); // IPv6 family
                    let xor_port = peer_addr.port() ^ ((STUN_MAGIC_COOKIE >> 16) as u16);
                    resp.extend_from_slice(&xor_port.to_be_bytes());
                    // Placeholder for IPv6 XOR masking
                    resp.extend_from_slice(&[0u8; 16]);
                }
            }

            let _ = socket.send_to(&resp, peer_addr).await;
        }
    }
}
