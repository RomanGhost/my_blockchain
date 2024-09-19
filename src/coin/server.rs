use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

type Clients = Arc<Mutex<HashMap<String, ClientData>>>;

#[derive(Clone)]
struct ClientData {
    stream: Arc<Mutex<TcpStream>>,
}

pub struct Server {
    clients: Clients,
    threads: Vec<JoinHandle<()>>, // Храним JoinHandle для каждого потока
}

impl Server {
    pub fn new() -> Server {
        let clients: Clients = Arc::new(Mutex::new(HashMap::new()));
        let threads: Vec<JoinHandle<()>> = vec![]; // Создаем вектор для JoinHandle
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

                    // Создаем новый поток для обработки клиента
                    let handle = thread::spawn(move || {
                        let handler = ClientHandler::new(stream, clients);
                        handler.handle();
                    });

                    // Сохраняем JoinHandle вместо самого потока
                    self.threads.push(handle);
                }
                Err(e) => eprintln!("Error accepting connection: {}", e),
            }
        }
    }
}

struct ClientHandler {
    clients: Clients,
    stream: Arc<Mutex<TcpStream>>,
    peer_addr: String,
}

impl ClientHandler {
    fn new(stream: TcpStream, clients: Clients) -> Self {
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

    fn handle(mut self) {
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
            match reader.read_line(&mut buffer) {
                Ok(0) => {
                    println!("Client disconnected: {}", self.peer_addr);
                    break;
                }
                Ok(_) => {
                    let message = format!("[{}]: {}\r", self.peer_addr, buffer.trim());
                    println!("{}", message);
                    // массовая рассылка
                    // broadcast_message(&message, &self.clients);
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

        self.cleanup();
    }

    fn cleanup(self) {
        let mut clients = match self.clients.lock() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error locking clients to remove disconnected client {}: {}", self.peer_addr, e);
                return;
            }
        };
        clients.remove(&self.peer_addr);
    }
}

// Реализация Drop для корректного завершения всех потоков
impl Drop for Server {
    fn drop(&mut self) {
        for handle in self.threads.drain(..) { // Очищаем вектор и вызываем join() для каждого потока
            if let Err(e) = handle.join() {
                eprintln!("Error joining thread: {:?}", e);
            }
        }
    }
}
