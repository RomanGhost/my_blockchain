<<<<<<< HEAD
use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};

// Определяем тип для хранения данных о клиентах
type Clients = Arc<Mutex<HashMap<u64, ClientData>>>;

// Структура для хранения информации о клиенте
// Теперь мы используем Arc<Mutex<TcpStream>> для потокобезопасного доступа к TcpStream
#[derive(Clone)]
struct ClientData {
    stream: Arc<Mutex<TcpStream>>,
    nickname: String,
}

fn main() {
    // Создаем и привязываем TcpListener к адресу на локальном хосте
    let listener = TcpListener::bind("localhost:7878").expect("Couldn't bind to address");
    // Создаем Arc<Mutex<>> для потокобезопасного хранения данных о клиентах
    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));
    let mut id_counter = 0; // Счетчик уникальных идентификаторов клиентов

    println!("Chat server running on port 7878");

    // Основной цикл для приема входящих соединений
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // Присваиваем уникальный идентификатор новому клиенту
                let id = id_counter;
                id_counter += 1;
                let clients = Arc::clone(&clients); // Клонируем Arc для передачи в новый поток

                // Создаем новый поток для обработки клиента
                thread::spawn(move || {
                    handle_client(stream, clients, id);
                });
            }
            Err(e) => eprintln!("Error accepting connection: {}", e),
        }
    }
}

// Функция для обработки соединения с клиентом
fn handle_client(stream: TcpStream, clients: Clients, id: u64) {
    // Получаем IP-адрес клиента
    let peer_addr = stream.peer_addr().expect("Couldn't get peer address");
    println!("Client connected: {}", peer_addr);

    let stream = Arc::new(Mutex::new(stream)); // Оборачиваем TcpStream в Arc<Mutex<>>
    let mut reader = BufReader::new(stream.lock().unwrap().try_clone().expect("Couldn't clone stream")); // Создаем BufReader для чтения из TcpStream
    let mut buffer = String::new(); // Буфер для чтения данных

    // Отправляем клиенту запрос на ввод ника
    if let Err(e) = writeln!(stream.lock().unwrap(), "Welcome to the chat server! Please enter your nickname:\r") {
        eprintln!("Error sending nickname prompt: {}", e);
        return; // Если не удалось отправить запрос, завершаем выполнение
    }

    // Читаем никнейм клиента
    buffer.clear();
    if reader.read_line(&mut buffer).is_err() {
        eprintln!("Error reading nickname");
        return; // Если не удалось прочитать никнейм, завершаем выполнение
    }
    let nickname = buffer.trim().to_string(); // Убираем пробелы и сохраняем никнейм

    // Сохраняем информацию о клиенте в хранилище
    //Определяем время жизни блокировки внутри блока
    {
        let mut clients = clients.lock().unwrap(); // Получаем доступ к хранилищу клиентов
        clients.insert(id, ClientData {
            stream: Arc::clone(&stream), // Сохраняем ссылку на поток
            nickname,
        });
    }

    // Основной цикл для обработки сообщений от клиента
    loop {
        buffer.clear();
        match reader.read_line(&mut buffer) {
            Ok(0) => {
                // Если клиент закрыл соединение (EOF)
                println!("Client disconnected: {}", peer_addr);
                break; // Выходим из цикла
            }
            Ok(_) => {
                // Формируем сообщение для рассылки
                let message = format!("[{}]: {}\r", get_nickname(&clients, id), buffer.trim());
                broadcast_message(&message, &clients); // Отправляем сообщение всем клиентам
            }
            Err(e) => {
                eprintln!("Error reading from client: {}", e);
                break; // Выходим из цикла при ошибке чтения
            }
        }
    }

    // Удаляем клиента из хранилища при отключении
    let mut clients = clients.lock().unwrap(); // Получаем доступ к хранилищу клиентов
    clients.remove(&id); // Удаляем клиента по его идентификатору
}

// Функция для получения ника клиента по его идентификатору
fn get_nickname(clients: &Clients, id: u64) -> String {
    let clients = clients.lock().unwrap(); // Получаем доступ к хранилищу клиентов
    clients.get(&id).map_or("Unknown".to_string(), |client| client.nickname.clone())
}

// Функция для рассылки сообщений всем подключенным клиентам
fn broadcast_message(message: &str, clients: &Clients) {
    let clients = clients.lock().unwrap(); // Получаем доступ к хранилищу клиентов
    for client in clients.values() {
        // Отправляем сообщение каждому клиенту
        if let Err(e) = writeln!(client.stream.lock().unwrap(), "{}", message) {
            eprintln!("Error sending message to client {}", e);
=======
mod coin;

use std::io::{self, Write, BufRead};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use crate::coin::connection::ConnectionPool;
use crate::coin::peers::P2PProtocol;
use crate::coin::server::Server;

fn get_input_text(info_text: &str) -> String {
    let mut input = String::new();
    print!("{}: ", info_text);
    io::stdout().flush().unwrap(); // Очищаем буфер, чтобы текст сразу отобразился в консоли
    match io::stdin().read_line(&mut input) {
        Ok(_) => input.trim().to_string(),
        Err(e) => {
            eprintln!("Error reading input: {}", e);
            String::new()
>>>>>>> main
        }
    }
}

fn main() {
    // Инициализация общего пула соединений
    let connection_pool = Arc::new(Mutex::new(ConnectionPool::new()));

    // Инициализация протокола для работы с пирами
    let p2p_protocol = Arc::new(Mutex::new(P2PProtocol::new(connection_pool.clone())));

    // Создание и запуск сервера
    let mut server = Server::new(connection_pool.clone(), p2p_protocol.clone());
    let server_clone = Arc::new(Mutex::new(server.clone()));
    let server_address = get_input_text("Введите адрес сервера (например, 127.0.0.1:7878)");

    // Запуск сервера в отдельном потоке
    let server_thread = thread::spawn({
        let server_address = server_address.clone();
        move || {
            server.run(&server_address);
        }
    });

    // Небольшая пауза для корректного запуска сервера
    thread::sleep(Duration::from_secs(1));

    loop {
        // Чтение ввода пользователя для команды
        println!("\nДоступные команды:");
        println!("1. Подключиться к другому серверу (формат: connect <IP>:<port>)");
        println!("2. Вещать сообщение всем пирами (broadcast <сообщение>)");
        println!("3. Выйти (exit)");

        let input = get_input_text("Введите команду");

        if input.starts_with("connect") {
            // Разбираем команду подключения
            let parts: Vec<&str> = input.split_whitespace().collect();
            if parts.len() == 2 {
                let address = parts[1];
                let address_parts: Vec<&str> = address.split(':').collect();
                if address_parts.len() == 2 {
                    let ip = address_parts[0];
                    if let Ok(port) = address_parts[1].parse::<u16>() {
                        // Подключаемся к другому серверу
                        server_clone.try_lock().unwrap().connect(ip, port);
                        // p2p_protocol.connect_to_peer(ip, port);
                    } else {
                        println!("Некорректный порт: {}", address_parts[1]);
                    }
                } else {
                    println!("Некорректный формат адреса. Используйте: connect <IP>:<port>");
                }
            } else {
                println!("Неверное количество аргументов. Используйте: connect <IP>:<port>");
            }
        } else if input.starts_with("broadcast") {
            // Разбираем команду вещания
            let mut parts: Vec<&str> = input.split_whitespace().collect();
            if parts.len() > 1 {
                let message = input.clone();
                // Вещаем сообщение всем подключенным пирами
                p2p_protocol.lock().unwrap().broadcast(&message);
            } else {
                println!("Сообщение не может быть пустым. Используйте: broadcast <сообщение>");
            }
        } else if input == "exit" {
            // Выходим из программы
            println!("Выход из программы.");
            break;
        } else {
            println!("Неверная команда.");
        }
    }

    // Ожидание завершения потока сервера
    server_thread.join().unwrap();
}