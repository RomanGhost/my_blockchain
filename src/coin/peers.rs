
use std::sync::{Arc, Mutex};
use std::io::Write;
use std::net::TcpStream;
use crate::coin::connection::ConnectionPool;

pub struct P2PProtocol {
    connection_pool: Arc<Mutex<ConnectionPool>>,
}

impl P2PProtocol {
    pub fn new(connection_pool: Arc<Mutex<ConnectionPool>>) -> Self {
        P2PProtocol {
            connection_pool,
        }
    }

    pub fn handle_message(&self, message: &str, peer_address: &str, stream: &mut TcpStream) {
        if message.contains("ping") {
            self.handle_ping(peer_address, stream);
        } else if message.contains("broadcast") {
            // Обрабатываем команду для вещания
            self.handle_broadcast(message);
        } else if message.contains("block") {
            self.handle_block(message, stream);
        } else if message.contains("transaction") {
            self.handle_transaction(message, stream);
        } else if message.contains("peers") {
            self.handle_peers(stream);
        }
    }

    fn handle_ping(&self, peer_address: &str, stream: &mut TcpStream) {
        println!("Handling ping from: {}", peer_address);
        let response = format!("pong from {}", peer_address);
        stream.write_all(response.as_bytes()).unwrap();
    }

    fn handle_broadcast(&self, message: &str) {
        // Вызываем функцию broadcast для передачи сообщения всем подключенным пирами
        println!("Broadcasting message: {}", message);
        let mut connection_pool = self.connection_pool.lock().unwrap();
        connection_pool.broadcast(message);
    }

    fn handle_block(&self, message: &str, stream: &mut TcpStream) {
        println!("Handling block: {}", message);
        stream.write_all(message.as_bytes()).unwrap();
    }

    fn handle_transaction(&self, message: &str, stream: &mut TcpStream) {
        println!("Handling transaction: {}", message);
        stream.write_all(message.as_bytes()).unwrap();
    }

    fn handle_peers(&self, stream: &mut TcpStream) {
        let connection_pool = self.connection_pool.lock().unwrap();
        let peer_addresses = connection_pool.get_peer_addresses();
        let peers_list = peer_addresses.join(", ");
        let response = format!("Peers: {}", peers_list);
        stream.write_all(response.as_bytes()).unwrap();
    }
}
