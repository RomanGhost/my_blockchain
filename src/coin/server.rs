use std::{
    collections::HashMap,
    net::TcpListener,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    io::Write,
};
use std::net::TcpStream;
use crate::coin::connection::ClientHandler;
use crate::coin::peers::Clients;

pub struct Server {
    clients: Clients,
    threads: Vec<JoinHandle<()>>,
}

impl Server {
    pub fn new() -> Server {
        let clients: Clients = Arc::new(Mutex::new(HashMap::new()));
        let threads: Vec<JoinHandle<()>> = vec![];
        Server { clients, threads }
    }

    pub fn run(&mut self, address: String) {
        let listener = match TcpListener::bind(address.clone()) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Couldn't bind to address: {}", e);
                return;
            }
        };
        // Потоки для обработки входящих соединений
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let clients = Arc::clone(&self.clients);

                    let handle = thread::spawn(move || {
                        let handler = ClientHandler::new(stream, clients);
                        handler.handle();
                    });

                    self.threads.push(handle);
                }
                Err(e) => eprintln!("Error accepting connection: {}", e),
            }
        }
    }

    /// Метод для массовой рассылки сообщения всем подключенным клиентам
    pub fn broadcast_message(&self, message: String) {
        let message = format!("{}\n\r", message.trim());

        let clients = match self.clients.try_lock() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error locking clients for broadcasting: {}", e);
                return;
            }
        };

        for (peer, client_data) in clients.iter() {
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

    pub fn connect_to_peer(&mut self, address: &str) {
        match TcpStream::connect(address) {
            Ok(stream) => {
                let clients = Arc::clone(&self.clients);

                let handle = thread::spawn(move || {
                    let handler = ClientHandler::new(stream, clients);
                    handler.handle();
                });

                println!("Connected to peer: {}", address);
                self.threads.push(handle);
            }
            Err(e) => {
                eprintln!("Couldn't connect to peer {}: {}", address, e);
            }
        }
    }
}

// Реализация Drop для корректного завершения всех потоков
impl Drop for Server {
    fn drop(&mut self) {
        for handle in self.threads.drain(..) {
            println!("Delete thread");
            if let Err(e) = handle.join() {
                eprintln!("Error joining thread: {:?}", e);
            }
        }
    }
}
