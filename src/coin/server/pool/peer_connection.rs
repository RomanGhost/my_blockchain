use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub struct PeerConnection {
    pub addr: SocketAddr,
    pub stream: Arc<Mutex<TcpStream>>,
    pub last_seen: Instant,
    pub buffer: String,
}