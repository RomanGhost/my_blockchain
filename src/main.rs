mod coin;

use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use crate::coin::blockchain::block::Block;
use crate::coin::blockchain::blockchain::Blockchain;
use crate::coin::message::r#type::Message;
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
    // Создание и запуск сервера
    let (mut server, rx_server) = Server::new();
    let p2p_protocol = server.get_peer_protocol();
    let p2p_protocol_message = server.get_peer_protocol();

    let server_clone = Arc::new(Mutex::new(server.clone()));
    let server_address = get_input_text("Введите адрес сервера (например, 127.0.0.1:7878)");

    // Запуск сервера в отдельном потоке
    let server_thread = thread::spawn({
        let server_address = server_address.clone();
        move || {
            server.run(&server_address);
        }
    });

    let mut blockchain:Blockchain = Blockchain::new();

    // получение сообщений от серверов
    let receiver_thread = thread::spawn(move || {
        for received in rx_server {
            let message = received;
            match message{
                Message::RequestLastNBlocksMessage(message) => {
                    let n = message.get_n();
                    let blocks = blockchain.get_last_n_blocks(n);

                    for block in blocks{
                        p2p_protocol_message.lock().unwrap().response_block(block, true, false);
                    }
                }
                Message::RequestBlocksBeforeMessage(message) => {
                    let time = message.get_time();
                    let blocks = blockchain.get_blocks_after(time);
                    for block in blocks{
                        p2p_protocol_message.lock().unwrap().response_block(block, true, false);
                    }
                }
                Message::ResponseBlockMessage(message) => {
                    let is_force_block = message.is_force();
                    if is_force_block {
                        blockchain.add_force_block(message.get_block());
                    }else{
                        blockchain.add_block(message.get_block());
                    }
                }
                Message::ResponseTransactionMessage(_) => {}
                Message::ResponseTextMessage(message) => {
                    let text = message.get_text();
                    println!("New message > {}", text);
                }
                (_)=>{eprintln!("Неизвестный тип сообщения");}
            };
        }
    });

    // Небольшая пауза для корректного запуска сервера
    thread::sleep(Duration::from_secs(1));
    let mut block_id = 0;

    loop {
        // Чтение ввода пользователя для команды
        println!("\nДоступные команды:");
        println!("1. Подключиться к другому серверу (формат: connect <IP>:<port>)");
        println!("2. Вещать сообщение всем пирами (broadcast <сообщение>)");
        println!("3. Добавить блок (block)");
        println!("4. Выйти (exit)");

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
                // Вещаем сообщение всем подключенным пирами
                p2p_protocol.lock().unwrap().response_text(message, false);
            } else {
                println!("Сообщение не может быть пустым. Используйте: broadcast <сообщение>");
            }
        } else if input == "exit" {
            println!("Выход из программы.");
            break;
        } else if input.starts_with("block"){
            let parts: Vec<&str> = input.split_whitespace().collect();
            let mut is_force_block = false;
            if parts.len() >= 2 {
                let part = parts[1];
                if part.trim().to_lowercase() == "true"{
                    is_force_block = true;
                }
            }
            let new_block = Block::new(block_id, vec![], format!("Hash: {block_id}"), 0);
            p2p_protocol.lock().unwrap().response_block(new_block, is_force_block, false);
            block_id += 1;
        }
        else {
            println!("Неверная команда.");
        }
    }
    receiver_thread.join().unwrap();
    server_thread.join().unwrap();
}