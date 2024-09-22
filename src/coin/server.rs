use std::{
    collections::HashMap,
    net::TcpListener,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
    io::Write,
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

        // Поток для периодической отправки "UPDATE SRV" сообщений
        let clients = Arc::clone(&self.clients);
        let update_handle = thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(60)); // Каждую минуту
                Server::broadcast_message(&clients, "UPDATE SRV");
            }
        });

        self.threads.push(update_handle);

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
    pub fn broadcast_message(clients: &Clients, message: &str) {
        let message = format!("{}\n\r", message);

        let clients_guard = clients.lock();
        if let Ok(clients) = clients_guard {
            for (peer, client_data) in clients.iter() {
                let mut stream = match client_data.stream.lock() {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Error locking stream for client {}: {}", peer, e);
                        continue;
                    }
                };

                if let Err(e) = stream.write_all(message.as_bytes()) {
                    eprintln!("Error sending message to client {}: {}", peer, e);
                }
            }
        } else {
            eprintln!("Failed to lock clients for broadcasting");
        }
    }

    /// Метод для отправки конкретного сообщения при событии
    pub fn notify_event(&self, event_message: &str) {
        Server::broadcast_message(&self.clients, event_message);
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
