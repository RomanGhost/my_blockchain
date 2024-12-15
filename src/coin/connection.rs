use std::collections::HashMap;
use std::net::TcpStream;
use std::io::Write;
use std::sync::{Arc, Mutex};

pub struct ConnectionPool {
    peers: HashMap<String, TcpStream>,
    buffer_size: usize,
}

impl ConnectionPool {
    pub fn new(buffer_size: usize) -> Self {
        ConnectionPool {
            peers: HashMap::new(),
            buffer_size,
        }
    }

    pub fn add_peer(&mut self, address: String, stream: TcpStream) {
        self.peers.insert(address.clone(), stream);
        println!("Added peer: {}", address);
    }

    pub fn remove_peer(&mut self, address: &str) {
        self.peers.remove(address);
        println!("Removed peer: {}", address);
    }

    pub fn get_alive_peers(&self) -> Vec<&TcpStream> {
        self.peers.values().collect()
    }

    pub fn get_peer_addresses(&self) -> Vec<String> {
        self.peers.keys().cloned().collect()
    }

    // Функция для вещания сообщения всем подключенным пирами
    pub fn broadcast(&mut self, message: &str) {
        // Создаем список адресов, которые нужно удалить
        let mut disconnected_peers = Vec::new();
        let buffer_size = self.buffer_size;

        let message = format!("{}\n", message);

        // Разбиваем сообщение на части в зависимости от размера буфера
        let mut start_index = 0;
        println!("Начинаем отправку сообщений");
        while start_index < message.len() {
            // Рассчитываем конечный индекс для среза
            let end_index = (start_index + buffer_size).min(message.len());
            let message_chunk = &message[start_index..end_index];
            println!("Само сообщение: {}", message_chunk);

            // Отправляем этот фрагмент всем пирами
            for (address, stream) in self.peers.iter_mut() {
                if let Err(e) = stream.write_all(message_chunk.as_bytes()) {
                    eprintln!("Failed to send message to {}: {}", address, e);
                    disconnected_peers.push(address.clone());
                }
            }

            // Увеличиваем начальный индекс для следующего фрагмента
            start_index += buffer_size;
        }
        println!("Сообщение отправлено");

        // Удаляем отключенные пиры
        for address in disconnected_peers {
            self.remove_peer(&address);
        }
    }


    pub fn get_buffer(&self) -> Vec<u8> {
        vec![0; self.buffer_size]
    }
}

// Обертка для многопоточного доступа
pub type SharedConnectionPool = Arc<Mutex<ConnectionPool>>;
