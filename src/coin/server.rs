use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, mpsc, Mutex};
use std::io::Read;
use std::sync::mpsc::Receiver;
use std::thread;
use crate::coin::connection::ConnectionPool;
use crate::coin::message::r#type::Message;
use crate::coin::peers::P2PProtocol;

#[derive(Clone)]
pub struct Server {
    connection_pool: Arc<Mutex<ConnectionPool>>,
    p2p_protocol: Arc<Mutex<P2PProtocol>>,
}

impl Server {
    pub fn new() -> (Self, Receiver<Message>) {
        let (tx, rx) = mpsc::channel();
        let connection_pool = Arc::new(Mutex::new(ConnectionPool::new()));
        // Инициализация протокола для работы с пирами
        let p2p_protocol = Arc::new(Mutex::new(P2PProtocol::new(connection_pool.clone(), tx)));

        (Server {
            connection_pool,
            p2p_protocol,
        }, rx)
    }

    pub fn run(&mut self, address: &str) {
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

    pub fn connect(&self, ip: &str, port: u16) {
        match TcpStream::connect((ip, port)) {
            Ok(mut stream) => {
                println!("Успешно подключено к {}:{}", ip, port);
                let connection_pool = self.connection_pool.clone();
                let p2p_protocol = self.p2p_protocol.clone();

                let peer_address = stream.peer_addr().unwrap().to_string();
                connection_pool.lock().unwrap().add_peer(peer_address.clone(), stream.try_clone().unwrap());

                thread::spawn(move || {
                    handle_connection(peer_address, &mut stream, connection_pool, p2p_protocol);
                });
            }
            Err(e) => {
                eprintln!("Не удалось подключиться: {:?}", e);
            }
        }
    }

    pub fn get_peer_protocol(&self) -> Arc<Mutex<P2PProtocol>> {
        self.p2p_protocol.clone()
    }
}

fn handle_connection(peer_address: String, stream: &mut TcpStream, connection_pool: Arc<Mutex<ConnectionPool>>, p2p_protocol: Arc<Mutex<P2PProtocol>>) {
    let mut buffer = [0; 512];

    p2p_protocol.lock().unwrap().request_first_message();

    loop {
        match stream.read(&mut buffer) {
            Ok(0) => {
                println!("Connection closed by peer: {}", peer_address);
                connection_pool.lock().unwrap().remove_peer(&peer_address);
                break;
            }
            Ok(_) => {
                let message = String::from_utf8_lossy(&buffer[..]);
                // println!("Received response_message from {}: {}", peer_address, response_message);
                //TODO Нормально обработать ошибки
                p2p_protocol.lock().unwrap().handle_message(&message);
                // connection_pool.lock().unwrap().broadcast(&response_message);
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
