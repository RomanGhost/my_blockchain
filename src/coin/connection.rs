use std::collections::HashMap;
use std::net::TcpStream;
use std::io::Write;
use std::sync::{Arc, Mutex};

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
        let message = format!("{}\n", message);

        // Создаем список адресов, которые нужно удалить
        let mut disconnected_peers = Vec::new();

        for (address, stream) in self.peers.iter_mut() {
            if let Err(e) = stream.write_all(message.as_bytes()) {
                eprintln!("Failed to send message to {}: {}", address, e);
                disconnected_peers.push(address.clone());
            }
        }

        // Удаляем отключенные пиры
        for address in disconnected_peers {
            self.remove_peer(&address);
        }
    }
}

// Обертка для многопоточного доступа
pub type SharedConnectionPool = Arc<Mutex<ConnectionPool>>;
