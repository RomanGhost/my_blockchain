use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{mpsc, Arc, Mutex};
use std::sync::mpsc::Sender;
use std::thread;
use std::time::{Duration, Instant};

use log::{error, info, warn};

use crate::coin::connection::ConnectionPool;
use crate::coin::message::r#type::Message;
use crate::coin::peers::P2PProtocol;

const HANDSHAKE_MESSAGE: &str = "NEW_CONNECT!";
const TIMEOUT: u64 = 300;
const BUFFER_SIZE: usize = 4096;

#[derive(Clone)]
pub struct Server {
    connection_pool: Arc<Mutex<ConnectionPool>>,
    p2p_protocol: Arc<Mutex<P2PProtocol>>,
}

impl Server {
    pub fn new(tx: Sender<Message>) -> Self {
        let connection_pool = Arc::new(Mutex::new(ConnectionPool::new(BUFFER_SIZE)));
        let p2p_protocol = Arc::new(Mutex::new(P2PProtocol::new(connection_pool.clone(), tx)));

        Server {
            connection_pool,
            p2p_protocol,
        }
    }

    pub fn run(&mut self, address: String) {
        let listener = match TcpListener::bind(address.clone()) {
            Ok(listener) => {
                info!("Successfully bound to address {}", address);
                listener
            }
            Err(e) => {
                error!("Could not bind to address {}: {}", address, e);
                return;
            }
        };

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let connection_pool = self.connection_pool.clone();
                    let p2p_protocol = self.p2p_protocol.clone();
                    let peer_address = stream.peer_addr().unwrap().to_string();

                    thread::spawn(move || {
                        if let Err(e) = handle_connection(peer_address, &mut stream, connection_pool, p2p_protocol, false) {
                            warn!("Failed to handle connection: {:?}", e);
                        }
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
            Ok(mut stream) => {
                info!("Successfully connected to {}:{}", ip, port);
                let connection_pool = self.connection_pool.clone();
                let p2p_protocol = self.p2p_protocol.clone();
                let peer_address = stream.peer_addr().unwrap().to_string();

                connection_pool.lock().unwrap().add_peer(peer_address.clone(), stream.try_clone().unwrap());

                thread::spawn(move || {
                    if let Err(e) = handle_connection(peer_address, &mut stream, connection_pool, p2p_protocol, true) {
                        warn!("Failed to handle connection: {:?}", e);
                    }
                });
            }
            Err(e) => {
                warn!("Cannot connect to: {:?}", e);
            }
        }
    }

    pub fn get_peer_protocol(&self) -> Arc<Mutex<P2PProtocol>> {
        self.p2p_protocol.clone()
    }
}

fn handle_connection(
    peer_address: String,
    stream: &mut TcpStream,
    connection_pool: Arc<Mutex<ConnectionPool>>,
    p2p_protocol: Arc<Mutex<P2PProtocol>>,
    is_connect: bool,
) -> Result<(), std::io::Error> {
    let mut buffer = vec![0; BUFFER_SIZE];
    let mut accumulated_data = String::new();
    let mut last_message_time = Instant::now();

    send_handshake(stream)?;

    while accumulated_data.trim() != HANDSHAKE_MESSAGE {
        read_and_handle_data(stream, &mut buffer, &mut accumulated_data, &peer_address, &connection_pool, &p2p_protocol, &mut last_message_time)?;
    }

    info!("Authorized client connected from {}", peer_address);
    connection_pool.lock().unwrap().add_peer(peer_address.clone(), stream.try_clone().unwrap());

    if is_connect {
        p2p_protocol.lock().unwrap().request_first_message();
    }

    monitor_inactivity(peer_address, stream, connection_pool, p2p_protocol, &mut last_message_time)
}

fn send_handshake(stream: &mut TcpStream) -> Result<(), std::io::Error> {
    stream.write_all(HANDSHAKE_MESSAGE.as_bytes())?;
    info!("Отправляем рукопожатие: {}", HANDSHAKE_MESSAGE);
    Ok(())
}

fn read_and_handle_data(
    stream: &mut TcpStream,
    buffer: &mut Vec<u8>,
    accumulated_data: &mut String,
    peer_address: &String,
    connection_pool: &Arc<Mutex<ConnectionPool>>,
    p2p_protocol: &Arc<Mutex<P2PProtocol>>,
    last_message_time: &mut Instant,
) -> Result<(), std::io::Error> {
    match stream.read(buffer) {
        Ok(n) => {
            if n == 0 {
                info!("Connection closed by peer: {}", peer_address);
                connection_pool.lock().unwrap().remove_peer(peer_address);
                return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "Connection closed by peer"));
            }

            *last_message_time = Instant::now();
            accumulated_data.push_str(&String::from_utf8_lossy(&buffer[..n]));

            Ok(while let Some((message, remaining_data)) = extract_message(accumulated_data) {
                info!("New message received: {}", message);
                p2p_protocol.lock().unwrap().handle_message(&message);
                *accumulated_data = remaining_data;
            })
        }
        Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
            // Timeout occurred, continue the loop to check inactivity
            Ok(())
        }
        Err(e) => {
            error!("Error reading from stream: {}", e);
            connection_pool.lock().unwrap().remove_peer(peer_address);
            Err(e)
        }
    }
}

fn monitor_inactivity(
    peer_address: String,
    stream: &mut TcpStream,
    connection_pool: Arc<Mutex<ConnectionPool>>,
    p2p_protocol: Arc<Mutex<P2PProtocol>>,
    last_message_time: &mut Instant,
) -> Result<(), std::io::Error> {
    loop {
        if last_message_time.elapsed() >= Duration::from_secs(TIMEOUT) {
            info!("Client {} inactive for 5 minutes, disconnecting", peer_address);
            connection_pool.lock().unwrap().remove_peer(&peer_address);
            break;
        }

        stream.set_read_timeout(Some(Duration::from_secs(5)))?;

        let mut buffer = vec![0; BUFFER_SIZE];
        match stream.read(&mut buffer) {
            Ok(n) => {
                if n == 0 {
                    info!("Connection closed by peer: {}", peer_address);
                    connection_pool.lock().unwrap().remove_peer(&peer_address);
                    break;
                }

                *last_message_time = Instant::now();
                let accumulated_data = String::from_utf8_lossy(&buffer[..n]);

                for message in accumulated_data.split('\n').filter_map(|line| line.trim().is_empty().then(|| line.to_string())) {
                    info!("New message received: {}", message);
                    p2p_protocol.lock().unwrap().handle_message(&message);
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                // Timeout occurred, continue the loop to check inactivity
                continue;
            }
            Err(e) => {
                error!("Error reading from stream: {}", e);
                connection_pool.lock().unwrap().remove_peer(&peer_address);
                break;
            }
        }
    }
    Ok(())
}

/// Извлекает одно сообщение из буфера данных, разделенных `\n`.
fn extract_message(data: &str) -> Option<(String, String)> {
    if let Some(index) = data.find('\n') {
        let message = data[..index].to_string();
        let remaining = data[(index + 1)..].to_string();
        Some((message, remaining))
    } else {
        None
    }
}
