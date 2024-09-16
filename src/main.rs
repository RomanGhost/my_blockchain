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
        }
    }
}
