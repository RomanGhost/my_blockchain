use std::collections::HashMap;
use std::net::TcpStream;
use std::io::Write;

pub struct ConnectionPool {
    peers: HashMap<String, TcpStream>,
}

impl ConnectionPool {
    pub fn new() -> Self {
        ConnectionPool {
            peers: HashMap::new(),
        }
    }

    pub fn add_peer(&mut self, address: String, stream: TcpStream) {
        self.peers.insert(address.clone(), stream);
        println!("Added peer: {}", address);
    }

    pub fn remove_peer(&mut self, address: &str) {
        self.peers.remove(address);
        println!("Removed peer: {}", address);
    }

    pub fn get_alive_peers(&self) -> Vec<&TcpStream> {
        self.peers.values().collect()
    }

    pub fn get_peer_addresses(&self) -> Vec<String> {
        self.peers.keys().cloned().collect()
    }

    // Функция для вещания сообщения всем подключенным пирами
    pub fn broadcast(&mut self, message: &str) {
        for (address, stream) in self.peers.iter_mut() {
            match stream.write_all(message.as_bytes()) {
                Ok(_) => {},//println!("Broadcasted response_message to {}", address),
                Err(e) => eprintln!("Failed to send response_message to {}: {}", address, e),
            }
        }
    }
}
