use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{mpsc, Arc, Mutex};
use std::sync::mpsc::Sender;
use std::thread;
use std::time::{Duration, Instant};

use log::{debug, error, info, warn};
use crate::coin::server::connection::ConnectionPool;
use crate::coin::server::protocol::message::r#type::Message;
use crate::coin::server::protocol::peers::P2PProtocol;
use crate::coin::server::errors::ServerError;

const HANDSHAKE_MESSAGE: &str = "NEW_CONNECT!\r\n";
const TIMEOUT_SECONDS: u64 = 600;
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

    pub fn run(&mut self, address: &str) -> io::Result<()> {
        let listener = TcpListener::bind(address)?;
        info!("Successfully bound to address {}", address);

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let connection_pool = self.connection_pool.clone();
                    let p2p_protocol = self.p2p_protocol.clone();
                    let peer_address = stream.peer_addr()?.to_string();

                    thread::spawn(move || {
                        if let Err(e) = handle_connection(&peer_address, &mut stream, &connection_pool, &p2p_protocol, false) {
                            warn!("Failed to handle connection: {:?}", e);
                        }
                    });
                }
                Err(e) => {
                    warn!("Failed to accept a connection: {:?}", e);
                }
            }
        }

        Ok(())
    }

    pub fn connect(&self, ip: &str, port: &str) -> io::Result<()> {
        let mut stream = TcpStream::connect(format!("{}:{}", ip, port))?;
        info!("Successfully connected to {}:{}", ip, port);

        let connection_pool = self.connection_pool.clone();
        let p2p_protocol = self.p2p_protocol.clone();
        let peer_address = stream.peer_addr()?.to_string();

        thread::spawn(move || {
            if let Err(e) = handle_connection(&peer_address, &mut stream, &connection_pool, &p2p_protocol, true) {
                warn!("Failed to handle connection: {:?}", e);
            }
        });

        Ok(())
    }

    pub fn get_peer_protocol(&self) -> Arc<Mutex<P2PProtocol>> {
        self.p2p_protocol.clone()
    }
}

fn handle_connection(
    peer_address: &str,
    stream: &mut TcpStream,
    connection_pool: &Arc<Mutex<ConnectionPool>>,
    p2p_protocol: &Arc<Mutex<P2PProtocol>>,
    is_connect: bool,
) -> Result<(), ServerError> {
    let mut last_message_time = Instant::now();

    send_handshake(stream)?;
    read_handshake(stream, peer_address, connection_pool, &mut last_message_time)?;

    info!("Authorized client connected from {}", peer_address);
    if !connection_pool.lock().unwrap().connection_exist(peer_address) {
        connection_pool.lock().unwrap().add_peer(peer_address.to_string(), stream.try_clone()?);
    }

    p2p_protocol.lock().unwrap().response_peers();
    if is_connect {
        p2p_protocol.lock().unwrap().request_first_message();
    }

    monitor_inactivity(peer_address, stream, connection_pool, p2p_protocol, &mut last_message_time)
}

fn send_handshake(stream: &mut TcpStream) -> io::Result<()> {
    stream.write_all(HANDSHAKE_MESSAGE.as_bytes())?;
    info!("Sent handshake: {}", HANDSHAKE_MESSAGE);
    Ok(())
}

fn read_handshake(
    stream: &mut TcpStream,
    peer_address: &str,
    connection_pool: &Arc<Mutex<ConnectionPool>>,
    last_message_time: &mut Instant,
) -> Result<(), ServerError> {
    info!("Waiting for handshake from {}", peer_address);
    let mut handshake_data = String::new();

    while handshake_data != HANDSHAKE_MESSAGE {
        if last_message_time.elapsed() >= Duration::from_secs(TIMEOUT_SECONDS) {
            info!("Client {} inactive for {} seconds, disconnecting", peer_address, TIMEOUT_SECONDS);
            connection_pool.lock().unwrap().remove_peer(peer_address);
            return Err(ServerError::Timeout(peer_address.to_string()));
        }

        stream.set_read_timeout(Some(Duration::from_secs(5)))?;

        let mut buffer = vec![0; BUFFER_SIZE];
        let n = stream.read(&mut buffer)?;
        if n == 0 {
            info!("Connection closed by peer: {}", peer_address);
            connection_pool.lock().unwrap().remove_peer(peer_address);
            return Err(ServerError::Timeout(peer_address.to_string()));
        }

        *last_message_time = Instant::now();
        handshake_data.push_str(&String::from_utf8_lossy(&buffer[..n]));
        debug!("Accumulated data: {}", handshake_data);

        if let Some((message, _)) = extract_message(&handshake_data) {
            if message == HANDSHAKE_MESSAGE.trim() {
                info!("Handshake successful with {}", peer_address);
                return Ok(());
            }
        }
    }

    Ok(())
}

fn monitor_inactivity(
    peer_address: &str,
    stream: &mut TcpStream,
    connection_pool: &Arc<Mutex<ConnectionPool>>,
    p2p_protocol: &Arc<Mutex<P2PProtocol>>,
    last_message_time: &mut Instant,
) -> Result<(), ServerError> {
    let mut accumulated_data = String::new();

    loop {
        if last_message_time.elapsed() >= Duration::from_secs(TIMEOUT_SECONDS) {
            info!("Client {} inactive for {} seconds, disconnecting", peer_address, TIMEOUT_SECONDS);
            connection_pool.lock().unwrap().remove_peer(peer_address);
            return Err(ServerError::Timeout(peer_address.to_string()));
        }

        stream.set_read_timeout(Some(Duration::from_secs(5)))?;

        let mut buffer = vec![0; BUFFER_SIZE];
        let n = stream.read(&mut buffer)?;
        if n == 0 {
            info!("Connection closed by peer: {}", peer_address);
            connection_pool.lock().unwrap().remove_peer(peer_address);
            return Err(ServerError::Timeout(peer_address.to_string()));
        }

        *last_message_time = Instant::now();
        accumulated_data.push_str(&String::from_utf8_lossy(&buffer[..n]));

        while let Some((message, remaining_data)) = extract_message(&accumulated_data) {
            info!("New message received: {}", message);
            p2p_protocol.lock().unwrap().handle_message(&message);
            accumulated_data = remaining_data;
        }
    }
}

/// Extracts a single message from the buffer, delimited by `\n`.
fn extract_message(data: &str) -> Option<(String, String)> {
    data.find('\n').map(|index| {
        let message = data[..index].trim().to_string();
        let remaining = data[(index + 1)..].to_string();
        (message, remaining)
    })
}