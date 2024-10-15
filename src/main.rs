use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use crossbeam::channel::{self, Receiver};

struct ChatServer {
    clients: Arc<Mutex<HashMap<String, TcpStream>>>, // Хранит клиентов
}

impl ChatServer {
    fn new() -> Self {
        ChatServer {
            clients: Arc::new(Mutex::new(HashMap::new())), // Инициализация хранилища клиентов
        }
    }

    fn handle_client(&self, stream: TcpStream) {
        let addr = match stream.peer_addr() {
            Ok(addr) => addr.to_string(),
            Err(e) => {
                println!("Не удалось получить IP-адрес клиента: {}", e);
                return;
            }
        };

        // Добавляем клиента в список
        {
            let mut clients = self.clients.lock().unwrap();
            clients.insert(addr.clone(), stream.try_clone().unwrap());
        }

        println!("Клиент {} подключен.", addr);
        let buf_reader = BufReader::new(stream);

        for line in buf_reader.lines() {
            match line {
                Ok(message) => {
                    println!("Получено сообщение от {}: {}", addr, message);
                    self.broadcast(&addr, &message);
                }
                Err(e) => {
                    println!("Ошибка при чтении сообщения от {}: {}", addr, e);
                    break;
                }
            }
        }

        // Удаляем клиента из списка при отключении
        {
            let mut clients = self.clients.lock().unwrap();
            clients.remove(&addr);
            println!("Клиент {} отключен.", addr);
        }
    }

    fn broadcast(&self, sender: &str, message: &str) {
        let clients = self.clients.lock().unwrap();
        for (addr, stream) in clients.iter() {
            // Не отправляем сообщение отправителю
            if addr != sender {
                let _ = stream.write_all(format!("{}: {}\n", sender, message).as_bytes());
            }
        }
    }
}

fn main() {
    let listener = TcpListener::bind("localhost:7878").unwrap();
    let server = Arc::new(ChatServer::new());
    println!("Сервер запущен на порту 7878");

    let (tx, rx): (channel::Sender<TcpStream>, Receiver<TcpStream>) = channel::bounded(100); // Ограничиваем размер канала

    let pool_size = 4; // Количество потоков в пуле

    // Создаем пул потоков
    for _ in 0..pool_size {
        let server_clone = Arc::clone(&server);
        let rx_clone = rx.clone();

        thread::spawn(move || {
            while let Ok(stream) = rx_clone.recv() {
                server_clone.handle_client(stream);
            }
        });
    }

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // Отправляем входящее соединение в пул
                if let Err(_) = tx.send(stream) {
                    println!("Ошибка при отправке соединения в пул потоков.");
                }
            }
            Err(e) => {
                println!("Ошибка при подключении клиента: {}", e);
            }
        }
    }
}
