use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use crate::coin::peers::{ClientData, Clients};

pub struct ClientHandler {
    clients: Clients,
    stream: Arc<Mutex<TcpStream>>, // Используем Arc<Mutex<TcpStream>> для совместного доступа из потоков
    peer_addr: String,
}

impl ClientHandler {
    pub fn new(stream: TcpStream, clients: Clients) -> Self {
        let peer_addr = match stream.peer_addr() {
            Ok(addr) => addr.to_string(),
            Err(e) => {
                eprintln!("Не удалось получить адрес узла: {}", e);
                String::new() // Если не удалось получить адрес, возвращаем пустую строку
            }
        };
        println!("Получено новое подключение!{}", peer_addr);
        Self {
            clients,
            stream: Arc::new(Mutex::new(stream)), // Оборачиваем TcpStream в Arc<Mutex>
            peer_addr,
        }
    }

    // Основная функция обработки клиента
    pub fn handle(self) {
        let mut buffer = String::new();
        let mut reader = BufReader::new(self.stream.lock().unwrap().try_clone().unwrap());

        // Добавляем клиента в общий список
        {
            let mut clients = match self.clients.lock() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Ошибка при блокировке клиентов: {}", e);
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

        // Читаем данные от клиента в цикле
        loop {
            buffer.clear();
            match reader.read_line(&mut buffer) {
                Ok(0) => {
                    println!("Клиент отключился: {}", self.peer_addr);
                    break;
                }
                Ok(_) => {
                    // Сообщение от клиента
                    let message = format!("[{}]: {}", self.peer_addr, buffer.trim());
                    println!("Получено новое сообщение: {}", message);
                    self.broadcast(message); // Рассылаем сообщение другим клиентам
                }
                Err(e) => {
                    eprintln!("Ошибка при чтении от клиента {}: {}", self.peer_addr, e);
                    break;
                }
            }
        }

        // Очищаем список клиентов при отключении
        self.cleanup();
    }

    // Функция рассылки сообщения всем клиентам, кроме отправителя
    pub fn broadcast(&self, message: String) {
        let message = format!("{}\n\r", message.trim());

        // Блокируем список клиентов для доступа
        let clients = match self.clients.lock() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Ошибка при блокировке клиентов для рассылки: {}", e);
                return;
            }
        };

        // Рассылаем сообщение всем клиентам, кроме отправителя
        for (peer, client_data) in clients.iter() {
            if peer != &self.peer_addr {
                let mut stream = match client_data.stream.lock() {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Ошибка при блокировке потока для клиента {}: {}", peer, e);
                        continue;
                    }
                };

                if let Err(e) = stream.write_all(message.as_bytes()) {
                    eprintln!("Ошибка отправки сообщения клиенту {}: {}", peer, e);
                }
            }
        }
    }

    // Функция для удаления клиента из списка при отключении
    fn cleanup(&self) {
        let mut clients = match self.clients.lock() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Ошибка при блокировке клиентов для удаления {}: {}", self.peer_addr, e);
                return;
            }
        };
        clients.remove(&self.peer_addr);
        println!("Клиент {} удален", self.peer_addr);
    }
}
