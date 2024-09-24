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
        }
    }
}

fn main() {
    // Инициализация общего пула соединений
    let connection_pool = Arc::new(Mutex::new(ConnectionPool::new()));

    // Инициализация протокола для работы с пирами
    let p2p_protocol = Arc::new(P2PProtocol::new(connection_pool.clone()));

    // Создание и запуск сервера
    let server = Server::new(connection_pool.clone(), p2p_protocol.clone());
    let server_address = get_input_text("Введите адрес сервера (например, 127.0.0.1:7878)");

    // Запуск сервера в отдельном потоке
    let server_thread = thread::spawn({
        let server_address = server_address.clone();
        move || {
            server.listen(&server_address);
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
                        p2p_protocol.connect_to_peer(ip, port);
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
            let parts: Vec<&str> = input.split_whitespace().collect();
            if parts.len() > 1 {
                let message = parts[1..].join(" ");
                // Вещаем сообщение всем подключенным пирами
                let mut pool = connection_pool.lock().unwrap();
                pool.broadcast(&message);
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