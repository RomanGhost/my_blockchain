use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::io::{Read, Write};
use std::thread;
use crate::coin::connection::ConnectionPool;
use crate::coin::peers::P2PProtocol;

pub struct Server {
    connection_pool: Arc<Mutex<ConnectionPool>>,
    p2p_protocol: Arc<P2PProtocol>,
}

impl Server {
    pub fn new(connection_pool: Arc<Mutex<ConnectionPool>>, p2p_protocol: Arc<P2PProtocol>) -> Self {
        Server {
            connection_pool,
            p2p_protocol,
        }
    }

    pub fn listen(&self, address: &str) {
        let listener = TcpListener::bind(address).expect("Could not bind to address");
        println!("Server listening on {}", address);

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let connection_pool = self.connection_pool.clone();
                    let p2p_protocol = self.p2p_protocol.clone();

                    let peer_address = stream.peer_addr().unwrap().to_string();
                    connection_pool.lock().unwrap().add_peer(peer_address.clone(), stream.try_clone().unwrap());

                    thread::spawn(move || {
                        handle_connection(peer_address, &mut stream, connection_pool, p2p_protocol);
                    });
                }
                Err(e) => {
                    eprintln!("Failed to accept a connection: {:?}", e);
                }
            }
        }
    }
}

fn handle_connection(peer_address: String, stream: &mut TcpStream, connection_pool: Arc<Mutex<ConnectionPool>>, p2p_protocol: Arc<P2PProtocol>) {
    let mut buffer = [0; 512];

    loop {
        match stream.read(&mut buffer) {
            Ok(0) => {
                println!("Connection closed by peer: {}", peer_address);
                connection_pool.lock().unwrap().remove_peer(&peer_address);
                break;
            }
            Ok(_) => {
                let message = String::from_utf8_lossy(&buffer[..]);
                println!("Received message from {}: {}", peer_address, message);
                p2p_protocol.handle_message(&message, &peer_address, stream);
                buffer = [0; 512];
            }
            Err(e) => {
                eprintln!("Error reading from stream: {:?}", e);
                connection_pool.lock().unwrap().remove_peer(&peer_address);
                break;
            }
        }
    }
}