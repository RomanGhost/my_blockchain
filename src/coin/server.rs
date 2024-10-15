use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{mpsc, Arc, Mutex};
use std::sync::mpsc::Receiver;
use std::thread;
use log::{error, info, warn};
use crate::coin::connection::ConnectionPool;
use crate::coin::message::r#type::Message;
use crate::coin::peers::P2PProtocol;
use crate::coin::thread_pool::ThreadPool;

/// TODO создать функцию clone внутри
/// TODO добавить мягкое завершение потоков
pub struct Server {
    thread_pool: ThreadPool,
    connection_pool: Arc<Mutex<ConnectionPool>>,
}

impl Server {
    pub fn new(num_threads: usize) -> Self {
        let connection_pool = Arc::new(Mutex::new(ConnectionPool::new(1024)));
        let thread_pool = ThreadPool::new(num_threads);

        Server {
            thread_pool,
            connection_pool,
        }
    }

    pub fn run(&self, address: String) {
        let listener = match TcpListener::bind(&address) {
            Ok(listener) => {
                info!("Successfully bound to address {}", address);
                listener
            }
            Err(e) => {
                error!("Could not bind to address {}: {}", address, e);
                return; // Останавливаем выполнение программы
            }
        };

        // Обработка входящих подключений
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let connection_pool = Arc::clone(&self.connection_pool);
                    let peer_address = stream.peer_addr().unwrap().to_string();

                    // Adding peer to the connection pool
                    connection_pool.lock().unwrap().add_peer(peer_address.clone(), stream.try_clone().unwrap());

                    // Execute the connection handling in the thread pool
                    let _ = self.thread_pool.execute(move || {
                        handle_connection(peer_address, stream, connection_pool);
                    });
                }
                Err(e) => {
                    warn!("Failed to accept a connection: {:?}", e);
                }
            }
        }
    }

    pub fn connect(&self, ip: &str, port: u16) {
        match TcpStream::connect((ip, port)) {
            Ok(stream) => {
                info!("Successfully connected to {}:{}", ip, port);
                let connection_pool = Arc::clone(&self.connection_pool);
                let peer_address = stream.peer_addr().unwrap().to_string();

                connection_pool.lock().unwrap().add_peer(peer_address.clone(), stream.try_clone().unwrap());

                // Execute the connection handling in the thread pool
                let _ = self.thread_pool.execute(move || {
                    handle_connection(peer_address, stream, connection_pool);
                });
            }
            Err(e) => {
                warn!("Cannot connect to: {:?}", e);
            }
        }
    }
}

// Function to handle individual connections
fn handle_connection(peer_address: String, mut stream: TcpStream, connection_pool: Arc<Mutex<ConnectionPool>>) {
    let mut buffer = vec![0; 1024];

    loop {
        match stream.read(&mut buffer) {
            Ok(0) => {
                info!("Connection closed by peer: {}", peer_address);
                let mut pool = connection_pool.lock().unwrap();
                pool.remove_peer(&peer_address);
                break;
            }
            Ok(n) => {
                println!("{}", &String::from_utf8_lossy(&buffer[..n]));
                // Handle received data (for demonstration, we just echo it back)
                if let Err(e) = stream.write_all(&buffer[..n]) {
                    error!("Failed to write to stream: {:?}", e);
                    break;
                }
            }
            Err(e) => {
                error!("Error reading from stream: {:?}", e);
                let mut pool = connection_pool.lock().unwrap();
                pool.remove_peer(&peer_address);
                break;
            }
        }
    }
}