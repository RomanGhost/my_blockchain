use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use crate::coin::peers::{ClientData, Clients};

pub struct ClientHandler {
    clients: Clients,
    stream: Arc<Mutex<TcpStream>>,
    peer_addr: String,
}

impl ClientHandler {
    pub fn new(stream: TcpStream, clients: Clients) -> Self {
        let peer_addr = match stream.peer_addr() {
            Ok(addr) => addr.to_string(),
            Err(e) => {
                eprintln!("Couldn't get peer address: {}", e);
                String::new() // Возвращаем пустой адрес, если не удалось получить
            }
        };
        Self {
            clients,
            stream: Arc::new(Mutex::new(stream)),
            peer_addr,
        }
    }

    pub fn handle(self) {
        let mut buffer = String::new();
        let mut reader = BufReader::new(self.stream.lock().unwrap().try_clone().unwrap());

        {
            let mut clients = match self.clients.lock() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error locking clients: {}", e);
                    return;
                }
            };
            clients.insert(
                self.peer_addr.clone(),
                ClientData {
                    stream: Arc::clone(&self.stream),
                },
            );
        }

        loop {
            buffer.clear();
            // Читаем сообщение от клиента
            match reader.read_line(&mut buffer) {
                Ok(0) => {
                    println!("Client disconnected: {}", self.peer_addr);
                    break;
                }
                Ok(_) => {
                    //Полученное сообщение от других клиентов
                    let message = format!("[{}]: {}", self.peer_addr, buffer.trim());
                    println!("Получено новое сообщение: {}", message);
                    // Массивная рассылка
                    self.broadcast(message);
                }
                Err(e) => {
                    eprintln!(
                        "Error reading from client {}: {}. Buffer: {}",
                        self.peer_addr, e, buffer
                    );
                    break;
                }
            }
        }
        // Удаляем клиента при отключении
        self.cleanup();
    }

    pub fn broadcast(&self, message: String) {
        let message = format!("{}\n\r", message.trim());

        let clients = match self.clients.lock() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error locking clients for broadcasting: {}", e);
                return;
            }
        };

        for (peer, client_data) in clients.iter() {
            if peer != &self.peer_addr {
                let mut stream = match client_data.stream.lock() {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Error locking stream for client {}: {}", peer, e);
                        continue;
                    }
                };

                if let Err(e) = stream.write_all(message.as_bytes()) {
                    eprintln!("Error writing message to client {}: {}", peer, e);
                }
            }
        }
    }

    fn cleanup(&self) {
        let mut clients = match self.clients.lock() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error locking clients to remove disconnected client {}: {}", self.peer_addr, e);
                return;
            }
        };
        clients.remove(&self.peer_addr);
        println!("Client {} removed", self.peer_addr);
    }
}
