use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;

#[derive(Debug)]
pub enum PoolMessage {
    NewPeer(SocketAddr, Arc<Mutex<TcpStream>>),
    PeerDisconnected(SocketAddr),
    BroadcastMessage(String),
    GetPeers(Sender<Vec<SocketAddr>>),
    PeerMessage(SocketAddr, String),
}