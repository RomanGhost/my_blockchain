mod coin;

use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use crate::coin::blockchain::blockchain::Blockchain;
use crate::coin::message::r#type::Message;
use crate::coin::server::Server;

fn get_input_text(info_text: &str) -> String {
    let mut input = String::new();
    print!("{}: ", info_text);
    io::stdout().flush().unwrap();
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
    let p2p_protocol_blockchain = server.get_peer_protocol();

    let server_clone = Arc::new(Mutex::new(server.clone()));
    let server_address = get_input_text("Введите адрес сервера (например, 127.0.0.1:7878)");

    // Запуск сервера в отдельном потоке
    let server_thread = thread::spawn({
        let server_address = server_address.clone();
        move || {
            server.run(&server_address);
        }
    });

    thread::sleep(Duration::from_secs(3));


    // Создаем основной блокчейн и майнинговый блокчейн
    let blockchain = Arc::new(Mutex::new(Blockchain::new()));
    let blockchain_message = Arc::clone(&blockchain);
    let mining_blockchain = Arc::clone(&blockchain);

    let mining_input = get_input_text("Запустить майнинг[y/n]:");
    if mining_input == "y" {

        // Поток для майнинга
        let miner_thread = thread::spawn(move || {
            loop {
                {
                    let mut chain = mining_blockchain.lock().unwrap();
                    // Если основной блокчейн пустой, создаем генезис-блок
                    if chain.len() == 0 {
                        chain.create_first_block();
                    } else {
                        // Запуск майнинга на основе последнего блока
                        chain.proof_of_work();
                    }

                    if let Ok(last_block) = chain.get_last_block() {
                        p2p_protocol_blockchain.lock().unwrap().response_block(last_block, false, false)
                    }
                }
                // Добавляем задержку перед следующей попыткой майнинга
                thread::sleep(Duration::from_secs(1));
            }
        });
    }

    // Поток для получения сообщений
    let receiver_thread = thread::spawn(move || {
        for received in rx_server {
            let message = received;
            match message {
                Message::RequestLastNBlocksMessage(message) => {
                    let n = message.get_n();
                    let blocks = blockchain.lock().unwrap().get_last_n_blocks(n);

                    for block in blocks {
                        p2p_protocol_message.lock().unwrap().response_block(block, true, false);
                        thread::sleep(Duration::from_millis(50));

                    }
                }
                Message::RequestBlocksBeforeMessage(message) => {
                    let time = message.get_time();
                    let blocks = blockchain.lock().unwrap().get_blocks_after(time);
                    for block in blocks {
                        p2p_protocol_message.lock().unwrap().response_block(block, true, false);
                    }
                }
                Message::ResponseBlockMessage(message) => {
                    let is_force_block = message.is_force();
                    let block = message.get_block();
                    let mut chain = blockchain.lock().unwrap();
                    if is_force_block {
                        chain.add_force_block(block);
                    } else {
                        chain.add_block(block);
                    }
                }
                Message::ResponseTransactionMessage(_) => {}
                Message::ResponseTextMessage(message) => {
                    let text = message.get_text();
                    println!("New message > {}", text);
                }
                _ => {
                    eprintln!("Неизвестный тип сообщения");
                }
            };
        }
    });

    loop {
        // Чтение ввода пользователя для команды
        println!("\nДоступные команды:");
        println!("1. Подключиться к другому серверу (формат: connect <IP>:<port>)");
        println!("2. Вещать сообщение всем пирами (broadcast <сообщение>)");
        println!("3. Добавить блок вручную (block)");
        println!("4. Выйти (exit)");

        let input = get_input_text("Введите команду");
        if input.starts_with("connect") {
            let parts: Vec<&str> = input.split_whitespace().collect();
            if parts.len() == 2 {
                let address = parts[1];
                let address_parts: Vec<&str> = address.split(':').collect();
                if address_parts.len() == 2 {
                    let ip = address_parts[0];
                    if let Ok(port) = address_parts[1].parse::<u16>() {
                        server_clone.lock().unwrap().connect(ip, port);
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
            let parts: Vec<&str> = input.split_whitespace().collect();
            if parts.len() > 1 {
                let message = parts[1..].join(" ");
                p2p_protocol.lock().unwrap().response_text(message, false);
            } else {
                println!("Сообщение не может быть пустым. Используйте: broadcast <сообщение>");
            }
        } else if input == "exit" {
            println!("Выход из программы.");
            break;
        } else if input.starts_with("blockchain") {
            // let chain = blockchain_message.lock().unwrap().chain.clone();
            println!("Заглушка...");
        } else {
            println!("Неверная команда.");
        }
    }

    receiver_thread.join().unwrap();
    // miner_thread.join().unwrap();
    server_thread.join().unwrap();
}
