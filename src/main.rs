mod coin;

use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use crate::coin::blockchain::blockchain::Blockchain;
use crate::coin::server::Server;
use crate::coin::message;
use crate::coin::message::MessageType;

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
    // Создание и запуск сервера
    let (mut server, rx) = Server::new();
    let p2p_protocol = server.get_peer_protocol();

    let server_clone = Arc::new(Mutex::new(server.clone()));
    let server_address = get_input_text("Введите адрес сервера (например, 127.0.0.1:7878)");

    // Запуск сервера в отдельном потоке
    let server_thread = thread::spawn({
        let server_address = server_address.clone();
        move || {
            server.run(&server_address);
        }
    });

    let blockchain = Blockchain::new();

    // получение сообщений от серверов
    let receiver_thread = thread::spawn(move || {
        for received in rx {
            let message_type = received.get_type();
            println!("> Received: {:?}{:?}", received, message_type);
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
                let text_message = message::TextMessage::new(message);
                let message = message::Message::TextMessage(text_message);

                // Вещаем сообщение всем подключенным пирами
                p2p_protocol.lock().unwrap().broadcast(message);
            } else {
                println!("Сообщение не может быть пустым. Используйте: broadcast <сообщение>");
            }
        } else if input == "exit" {
            println!("Выход из программы.");
            break;
        } else {
            println!("Неверная команда.");
        }
    }
    receiver_thread.join().unwrap();
    server_thread.join().unwrap();
}