use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{mpsc, Arc, Mutex};
use std::sync::mpsc::Sender;
use std::thread;
use std::time::{Duration, Instant};

use log::{debug, error, info, warn};
use serde::de::Unexpected::Str;
use crate::coin::server::connection::ConnectionPool;
use crate::coin::server::protocol::message::r#type::Message;
use crate::coin::server::protocol::peers::P2PProtocol;
use crate::coin::server::errors::ServerError;

const HANDSHAKE_MESSAGE: &str = "NEW_CONNECT!\r\n";
const TIMEOUT: u64 = 600;
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

                    thread::spawn(move || {
                        if let Err(e) = handle_connection(&mut stream, connection_pool, p2p_protocol, false) {
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

    pub fn connect(&self, ip: &str, port: &str) {

        match TcpStream::connect(format!("{}:{}", ip, port)) {
            Ok(mut stream) => {
                info!("Successfully connected to {}:{}", ip, port);
                let connection_pool = self.connection_pool.clone();
                let p2p_protocol = self.p2p_protocol.clone();

                thread::spawn(move || {
                    if let Err(e) = handle_connection(&mut stream, connection_pool, p2p_protocol, true) {
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
    stream: &mut TcpStream,
    connection_pool: Arc<Mutex<ConnectionPool>>,
    p2p_protocol: Arc<Mutex<P2PProtocol>>,
    is_connect: bool,
) -> Result<(), ServerError> {
    let mut last_message_time = Instant::now();

    let peer_address = stream.peer_addr().unwrap().to_string();

    send_handshake(stream)?;
    match read_handshake(stream, &connection_pool, &mut last_message_time) {
        Ok(_) => {
            info!("Authorized client connected from {}", peer_address);
            if !connection_pool.lock().unwrap().connection_exist(peer_address.clone().as_ref()) {
                connection_pool.lock().unwrap().add_peer(peer_address.clone(), stream.try_clone().unwrap());
            }
        }
        Err(e) => {
            info!("Error {}", e);
            return Err(e)
        }
    }

    {
        p2p_protocol.lock().unwrap_or_else(|e| {
            error!("Mutex connection_pool отравлен: {:?}", e);
            e.into_inner()
        }).response_peers();
    }
    if is_connect {
        p2p_protocol.lock().unwrap().request_first_message();
    }
    let _ = monitor_inactivity(stream, connection_pool, p2p_protocol, &mut last_message_time);

    Ok(())
}

fn send_handshake(stream: &mut TcpStream) -> Result<(), std::io::Error> {
    stream.write_all(HANDSHAKE_MESSAGE.as_bytes())?;
    info!("Отправляем рукопожатие: {}", HANDSHAKE_MESSAGE);
    Ok(())
}

fn read_handshake(
    stream: &mut TcpStream,
    connection_pool: &Arc<Mutex<ConnectionPool>>,
    last_message_time: &mut Instant,
) -> Result<(), ServerError> {
    info!("Wait handshake");
    let mut handsnake:String = String::new();
    let peer_address = stream.peer_addr().unwrap().to_string();

    while handsnake != HANDSHAKE_MESSAGE {
        if last_message_time.elapsed() >= Duration::from_secs(TIMEOUT) {
            info!("Client {} inactive for 5 minutes, disconnecting", peer_address);
            { connection_pool.lock().unwrap().remove_peer(peer_address.clone().as_ref()); }
            return Err(ServerError::Timeout(peer_address.clone()));
        }

        stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();

        let mut buffer = vec![0; BUFFER_SIZE];
        match stream.read(&mut buffer) {
            Ok(n) => {
                if n == 0 {
                    info!("Connection closed by peer: {}", peer_address);
                    { connection_pool.lock().unwrap().remove_peer(peer_address.as_ref()); }
                    return Err(ServerError::Timeout(peer_address.clone()));
                }

                *last_message_time = Instant::now();
                handsnake.push_str(&String::from_utf8_lossy(&buffer[..n]));
                debug!("Accum data: {}", handsnake);
                while let Some((message, _)) = extract_message(&handsnake) {
                    info!("New message received: {}", message);
                    if message+"\n" == HANDSHAKE_MESSAGE{
                        return Ok(());
                    }
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                // Timeout occurred, continue the loop to check inactivity
                continue;
            }
            Err(e) => {
                error!("Error reading from stream: {}", e);
                {
                    connection_pool.lock().unwrap().remove_peer(peer_address.as_ref());
                }
                return Err(ServerError::Io(e));
            }
        }

    }
    info!("Handshake is ok");
    Ok(())
}

fn monitor_inactivity(
    stream: &mut TcpStream,
    connection_pool: Arc<Mutex<ConnectionPool>>,
    p2p_protocol: Arc<Mutex<P2PProtocol>>,
    last_message_time: &mut Instant,
) -> Result<(), ServerError> {
    let mut accumulated_data:String=String::new();

    let peer_address = stream.peer_addr().unwrap().to_string();

    loop {
        if last_message_time.elapsed() >= Duration::from_secs(TIMEOUT) {
            info!("Client {} inactive for 5 minutes, disconnecting", peer_address);
            {
                connection_pool.lock().unwrap().remove_peer(peer_address.clone().as_ref());
            }
            return Err(ServerError::Timeout(peer_address.clone()));
        }

        stream.set_read_timeout(Some(Duration::from_secs(5)))?;

        let mut buffer = vec![0; BUFFER_SIZE];
        match stream.read(&mut buffer) {
            Ok(n) => {
                if n == 0 {
                    info!("Connection closed by peer: {}", peer_address);
                    {
                        connection_pool.lock().unwrap().remove_peer(peer_address.clone().as_ref());
                    }
                    return Err(ServerError::Timeout(peer_address.clone()));
                }

                *last_message_time = Instant::now();
                accumulated_data.push_str(&String::from_utf8_lossy(&buffer[..n]));
                while let Some((message, remaining_data)) = extract_message(&accumulated_data) {
                    info!("New message received: {}", message);
                    {
                        p2p_protocol.lock().unwrap().handle_message(&message);
                    }
                    accumulated_data = remaining_data;
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut || e.kind() == std::io::ErrorKind::WouldBlock => {
                // Нет данных для чтения, можно просто продолжить цикл
                continue;
            }
            Err(e) => {
                error!("Error reading from main stream: {}", e);
                connection_pool.lock().unwrap().remove_peer(peer_address.as_ref());
                return Err(ServerError::Io(e));
            }
        }
    }
}

/// Извлекает одно сообщение из буфера данных, разделенных `\n`.
fn extract_message(data: &str) -> Option<(String, String)> {
    if let Some(index) = data.find("\n") {
        let message = data[..index].to_string();
        let remaining = data[(index + 1)..].to_string();
        Some((message, remaining))
    } else {
        None
    }
}