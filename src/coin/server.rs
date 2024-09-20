use std::{
    collections::HashMap,
    net::TcpListener,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};
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

    pub fn run(&mut self, address: &str) {
        let listener = match TcpListener::bind(address) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Couldn't bind to address: {}", e);
                return;
            }
        };

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