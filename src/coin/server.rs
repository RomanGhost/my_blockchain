use std::{
    collections::HashMap,
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    io::{BufRead, BufReader, Write},
};
use crate::coin::connection::ClientHandler;
use crate::coin::peers::{ClientData, Clients};

pub struct Server {
    clients: Clients,
    peers: Vec<String>,  // Список подключенных узлов
    threads: Vec<JoinHandle<()>>,
}

impl Server {
    pub fn new() -> Server {
        let clients: Clients = Arc::new(Mutex::new(HashMap::new())); // Общий список клиентов
        let threads: Vec<JoinHandle<()>> = vec![]; // Вектор для хранения потоков
        let peers: Vec<String> = vec![]; // Список для хранения подключенных узлов
        Server { clients, peers, threads }
    }

    // Метод для запуска сервера
    pub fn run(&mut self, address: String) {
        let listener = match TcpListener::bind(&address) {
            Ok(listener) => listener,
            Err(e) => {
                eprintln!("Не удалось привязаться к адресу: {}", e);
                return;
            }
        };

        println!("Сервер запущен и слушает на {}", address);

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("Подключился новый клиент");
                    let clients = Arc::clone(&self.clients);
                    thread::spawn(move || {
                        let handler = ClientHandler::new(stream, clients);
                        handler.handle();
                    });
                }
                Err(e) => eprintln!("Ошибка при принятии соединения: {}", e),
            }
        }
    }


    // Метод для рассылки сообщения всем клиентам и узлам
    pub fn broadcast_message(&self, message: String) {
        let message = format!("{}\n\r", message.trim());

        // Блокируем список клиентов для безопасного доступа из потоков
        let clients = self.clients.lock().expect("Не удалось заблокировать список клиентов");

        // Отправляем сообщение всем подключенным клиентам
        for (peer, client_data) in clients.iter() {
            let mut stream = client_data.stream.lock().expect("Не удалось заблокировать поток клиента");

            if let Err(e) = stream.write_all(message.as_bytes()) {
                eprintln!("Ошибка отправки сообщения клиенту {}: {}", peer, e);
            }
        }

        // Рассылаем сообщение всем подключенным узлам
        for peer_addr in &self.peers {
            match TcpStream::connect(peer_addr) {
                Ok(mut stream) => {
                    if let Err(e) = stream.write_all(message.as_bytes()) {
                        eprintln!("Ошибка отправки сообщения узлу {}: {}", peer_addr, e);
                    } else {
                        println!("Сообщение отправлено узлу: {}", peer_addr);
                    }
                }
                Err(e) => {
                    eprintln!("Не удалось подключиться к узлу {}: {}", peer_addr, e);
                }
            }
        }
        println!("Sended message: {}", message);
    }

    // Метод для подключения к другому узлу
    pub fn connect_to_peer(&mut self, address: &str) {
        match TcpStream::connect(address) {
            Ok(stream) => {
                let clients = Arc::clone(&self.clients); // Клонируем Arc для нового потока
                let peer_address = address.to_string();

                // Создаем новый поток для подключения
                let handle = thread::spawn(move || {
                    let handler = ClientHandler::new(stream, clients); // Создаем обработчик клиента
                    handler.handle(); // Обрабатываем соединение
                });

                println!("Подключен к узлу: {}", address);
                self.peers.push(peer_address); // Добавляем узел в список peers
                self.threads.push(handle); // Добавляем поток в вектор
            }
            Err(e) => {
                eprintln!("Не удалось подключиться к узлу {}: {}", address, e);
            }
        }
    }
}

// Реализация Drop для корректного завершения всех потоков
impl Drop for Server {
    fn drop(&mut self) {
        for handle in self.threads.drain(..) {
            println!("Удаление потока");
            if let Err(e) = handle.join() {
                eprintln!("Ошибка при завершении потока: {:?}", e);
            }
        }
    }
}
