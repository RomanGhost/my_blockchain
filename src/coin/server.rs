use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::sync::{mpsc, Arc, Mutex};
use std::sync::mpsc::Receiver;
use std::thread;
use crate::coin::connection::ConnectionPool;
use crate::coin::message::r#type::Message;
use crate::coin::peers::P2PProtocol;

/// TODO создать функцию clone внутри
/// TODO добавить мягкое завершение потоков
#[derive(Clone)]
pub struct Server {
    connection_pool: Arc<Mutex<ConnectionPool>>,
    p2p_protocol: Arc<Mutex<P2PProtocol>>,
}

impl Server {
    pub fn new() -> (Self, Receiver<Message>) {
        let (tx, rx) = mpsc::channel();
        let connection_pool = Arc::new(Mutex::new(ConnectionPool::new(1024)));
        let p2p_protocol = Arc::new(Mutex::new(P2PProtocol::new(connection_pool.clone(), tx)));

        (
            Server {
                connection_pool,
                p2p_protocol,
            },
            rx,
        )
    }

    pub fn run(&mut self, address: String) {
        println!("Server listening on {}", address);
        let listener = TcpListener::bind(address).expect("Could not bind to address");

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let connection_pool = self.connection_pool.clone();
                    let p2p_protocol = self.p2p_protocol.clone();

                    let peer_address = stream.peer_addr().unwrap().to_string();
                    connection_pool.lock().unwrap().add_peer(peer_address.clone(), stream.try_clone().unwrap());

                    thread::spawn(move || {
                        handle_connection(peer_address, &mut stream, connection_pool, p2p_protocol, false);
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
                    handle_connection(peer_address, &mut stream, connection_pool, p2p_protocol, true);
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


fn handle_connection(
    peer_address: String,
    stream: &mut TcpStream,
    connection_pool: Arc<Mutex<ConnectionPool>>,
    p2p_protocol: Arc<Mutex<P2PProtocol>>,
    is_connect: bool,
) {
    let mut lock_connection_pool = connection_pool.lock().unwrap();
    let mut buffer = lock_connection_pool.get_buffer();
    drop(lock_connection_pool);

    let mut accumulated_data = String::new(); // Строковый буфер для хранения неполных данных

    if is_connect {
        p2p_protocol.lock().unwrap().request_first_message();
    }

    loop {
        match stream.read(&mut buffer) {
            Ok(0) => {
                let mut lock_connection_pool = connection_pool.lock().unwrap();
                println!("Connection closed by peer: {}", peer_address);
                lock_connection_pool.remove_peer(&peer_address);
                break;
            }
            Ok(n) => {
                // Добавляем полученные данные в строковый буфер
                accumulated_data.push_str(&String::from_utf8_lossy(&buffer[..n]));

                // Обработка буфера построчно
                while let Some((message, remaining_data)) = extract_message(&accumulated_data) {
                    p2p_protocol.lock().unwrap().handle_message(&message);
                    accumulated_data = remaining_data;
                }
            }
            Err(e) => {
                let mut lock_connection_pool = connection_pool.lock().unwrap();
                eprintln!("Error reading from stream: {:?}", e);
                lock_connection_pool.remove_peer(&peer_address);
                break;
            }
        }
    }
}

/// Извлекает одно сообщение из буфера данных, разделенных `\n`.
/// Возвращает кортеж (сообщение, остаток данных).
fn extract_message(data: &str) -> Option<(String, String)> {
    if let Some(index) = data.find('\n') {
        // Находим первое сообщение и остаток
        let message = data[..index].to_string();
        let remaining = data[(index + 1)..].to_string(); // Пропускаем символ новой строки
        Some((message, remaining))
    } else {
        None
    }
}
