use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};
use std::collections::HashSet;
use rust_chat_server::ThreadPool;

type ClientList = Arc<Mutex<HashSet<String>>>; // Используем HashSet для хранения строк (идентификаторов клиентов)

fn main() {
    // Создание слушателя на порту 7878
    let listener = TcpListener::bind("localhost:7878").unwrap();
    let pool = ThreadPool::new(4); // Создаем пул потоков из 4 потоков

    let clients: ClientList = Arc::new(Mutex::new(HashSet::new())); // Инициализируем список клиентов

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let clients = Arc::clone(&clients); // Клонируем указатель на список клиентов

        pool.execute(move || {
            handle_connection(stream, clients);
        });
    }
}

// Функция для обработки подключения клиента
fn handle_connection(mut stream: TcpStream, clients: ClientList) {
    let client_address = match stream.peer_addr() {
        Ok(addr) => addr.to_string(),
        Err(_) => return,
    };
    println!("Клиент подключен: {}", client_address);

    let mut clients_guard = clients.lock().unwrap(); // Захватываем блокировку для списка клиентов
    clients_guard.insert(client_address.clone()); // Добавляем строку с адресом клиента в общий список
    drop(clients_guard); // Отпускаем блокировку

    let reader = BufReader::new(stream.try_clone().unwrap());

    // Отправляем сообщение о новом подключении
    broadcast_message(&clients, &format!("{} just joined", client_address));

    // Обработка сообщений от клиента
    for line in reader.lines() {
        let message = match line {
            Ok(msg) => msg,
            Err(_) => {
                break;
            }
        };

        // Проверка специальных команд
        if message == "/quit" {
            println!("{} disconnected", client_address);
            break;
        } else if message == "/list" {
            list_users(&mut stream, &clients);
        } else {
            broadcast_message(&clients, &format!("[{}] {}", client_address, message));
        }
    }

    // Удаляем клиента из списка при отключении
    let mut clients_guard = clients.lock().unwrap();
    clients_guard.remove(&client_address);
    drop(clients_guard);

    // Сообщаем другим пользователям об отключении клиента
    broadcast_message(&clients, &format!("{} has quit", client_address));
}

// Функция для отправки сообщения всем пользователям
fn broadcast_message(clients: &ClientList, message: &str) {
    let clients_guard = clients.lock().unwrap(); // Захватываем блокировку для списка клиентов

    for client_addr in clients_guard.iter() {
        println!("Отправка сообщения пользователю {}: {}", client_addr, message);
        // Здесь предполагается отправка сообщений, возможно, через сохраненные TcpStream,
        // или другой способ, обеспечивающий связь.
    }
}

// Функция для отправки списка всех пользователей
fn list_users(stream: &mut TcpStream, clients: &ClientList) {
    let clients_guard = clients.lock().unwrap();
    let users: Vec<String> = clients_guard.iter().cloned().collect();

    let response = format!("===\nConnected users:\n{}\n===", users.join("\n - "));
    let _ = stream.write_all(response.as_bytes());
}
