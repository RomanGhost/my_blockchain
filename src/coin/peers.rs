
use std::sync::{Arc, Mutex};
use std::io::Write;
use std::net::TcpStream;
use crate::coin::connection::ConnectionPool;

pub struct P2PProtocol {
    connection_pool: Arc<Mutex<ConnectionPool>>,
    last_message_id:u64,
}

impl P2PProtocol {
    pub fn new(connection_pool: Arc<Mutex<ConnectionPool>>) -> Self {
        P2PProtocol {
            connection_pool,
            last_message_id:0,
        }
    }

    pub fn handle_message(&mut self, message: &str, peer_address: &str, stream: &mut TcpStream) {
        let message = message.trim();
        let mut id_message = 0;
        if let Some(first_part) = message.split_whitespace().next() {
            // Пробуем преобразовать первую часть в число
            if let Ok(number) = first_part.parse::<u64>() {
                id_message = number;
            } else {
                eprintln!("Не удалось преобразовать в число: {first_part}, {message}");
            }
        }

        if id_message < self.last_message_id{
            return;
        }

        // TODO преобразовать это в функцию
        let parts: Vec<&str> = message.split_whitespace().collect();
        let text_message = parts[2..].join(" ");
        println!(">- Get message [{peer_address}]: {text_message}");

        if message.contains("ping") {
            self.handle_ping(peer_address, stream);
        } else if message.contains("broadcast") {
            self.handle_broadcast(message);
        } else if message.contains("block") {
            self.handle_block(message, stream);
        } else if message.contains("transaction") {
            self.handle_transaction(message, stream);
        } else if message.contains("peers") {
            self.handle_peers(stream);
        }
        self.last_message_id+=1;
    }

    fn handle_ping(&self, peer_address: &str, stream: &mut TcpStream) {
        println!("Handling ping from: {}", peer_address);
        let response = format!("pong from {}", peer_address);
        stream.write_all(response.as_bytes()).unwrap();
    }

    fn handle_broadcast(&mut self, message: &str) {
        // Вызываем функцию broadcast для передачи сообщения всем подключенным пирами

        let parts: Vec<&str> = message.split_whitespace().collect();
        let message = parts[1..].join(" ");
        // println!("Broadcasting message: {}", message);
        // message.
        self.broadcast(message.as_ref());
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

    pub fn broadcast(&mut self, message:&str){
        let new_message = format!("{} {}", self.last_message_id, message);
        let mut connection_pool = self.connection_pool.lock().unwrap();
        self.last_message_id += 1;
        connection_pool.broadcast(new_message.as_ref());
    }
}
