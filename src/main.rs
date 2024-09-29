mod coin;

use std::io::{self, Write};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;
use crate::coin::blockchain::blockchain::Blockchain;
use crate::coin::message::r#type::Message;
use crate::coin::server::Server;

// Функция для получения пользовательского ввода с сообщением.
fn get_input_text(info_text: &str) -> String {
    print!("{}: ", info_text);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => input.trim().to_string(),
        Err(e) => {
            eprintln!("Error reading input: {}", e);
            String::new()
        }
    }
}

fn main() {
    // Создаем и инициализируем сервер.
    let (mut server, rx_server) = Server::new();
    let server_clone = server.clone();
    let p2p_protocol = server.get_peer_protocol();

    // Ввод адреса сервера и запуск его в отдельном потоке.
    let server_address = get_input_text("Введите адрес сервера (например, 127.0.0.1:7878)");
    let server_thread = {
        let server_address = server_address.clone();
        thread::spawn(move || {
            server.run(&server_address);
        })
    };

    thread::sleep(Duration::from_secs(3)); // Небольшая задержка для инициализации сервера.

    // Создаем основной блокчейн и переменные синхронизации для майнинга.
    let blockchain = Arc::new(Mutex::new(Blockchain::new()));
    let mining_flag = Arc::new((Mutex::new(false), Condvar::new()));

    // Проверяем, хочет ли пользователь запустить майнинг.
    if get_input_text("Запустить майнинг[y/n]:") == "y" {
        let blockchain = Arc::clone(&blockchain);
        let mining_flag = Arc::clone(&mining_flag);
        let p2p_protocol = Arc::clone(&p2p_protocol);

        thread::spawn(move || {
            let (lock, cvar) = &*mining_flag;
            loop {
                let mut should_stop = lock.lock().unwrap();
                // Ожидаем, пока не будет разрешено продолжение майнинга.
                while *should_stop {
                    should_stop = cvar.wait(should_stop).unwrap();
                }

                {
                    let mut chain = blockchain.lock().unwrap();
                    let iteration_result = chain.proof_of_work();

                    // Если новый блок найден, отправляем его другим узлам.
                    if iteration_result {
                        if let Ok(last_block) = chain.get_last_block() {
                            p2p_protocol.lock().unwrap().response_block(last_block, false);
                        }
                    }
                }
                thread::sleep(Duration::from_millis(1));
            }
        });
    }

    // Поток для обработки сообщений, поступающих на сервер.
    let receiver_thread = {
        let blockchain = Arc::clone(&blockchain);
        let p2p_protocol = Arc::clone(&p2p_protocol);
        let mining_flag = Arc::clone(&mining_flag);

        thread::spawn(move || {
            for received in rx_server {
                match received {
                    Message::RequestLastNBlocksMessage(message) => {
                        let n = message.get_n();
                        let blocks = blockchain.lock().unwrap().get_last_n_blocks(n);

                        for block in blocks {
                            // println!("Send block id: {}", block.get_id());
                            p2p_protocol.lock().unwrap().response_block(block, true);
                            thread::sleep(Duration::from_millis(3));
                        }
                    }
                    Message::RequestBlocksBeforeMessage(message) => {
                        let time = message.get_time();
                        let blocks = blockchain.lock().unwrap().get_blocks_after(time);
                        for block in blocks {
                            p2p_protocol.lock().unwrap().response_block(block, true);
                        }
                    }
                    Message::ResponseBlockMessage(message) => {
                        let is_force_block = message.is_force();
                        let block = message.get_block();
                        let mut chain = blockchain.lock().unwrap();

                        // Остановка майнинга при получении нового блока.
                        {
                            let (lock, cvar) = &*mining_flag;
                            let mut stop_flag = lock.lock().unwrap();
                            *stop_flag = true;
                        }
                        println!("Mining stop");
                        //
                        println!("Add new block");
                        // Добавление нового блока.
                        if is_force_block {
                            println!("\tforce add");
                            chain.add_force_block(block);
                        } else {
                            println!("\tusual add");
                            chain.add_block(block);
                        }

                        // Сигнал к продолжению майнинга.
                        {
                            let (lock, cvar) = &*mining_flag;
                            let mut stop_flag = lock.lock().unwrap();
                            *stop_flag = false;
                            cvar.notify_all();
                        }
                        println!("Mining start");
                    }
                    Message::ResponseTransactionMessage(_) => {}
                    Message::ResponseTextMessage(message) => {
                        println!("Новое сообщение > {}", message.get_text());
                    }
                    _ => {
                        eprintln!("Неизвестный тип сообщения");
                    }
                }
            }
        })
    };

    // Основной цикл для обработки пользовательских команд.
    loop {
        println!("\nДоступные команды:");
        println!("1. Подключиться к другому серверу (connect <IP>:<port>)");
        println!("2. Вещать сообщение всем пирами (broadcast <сообщение>)");
        println!("3. Выйти (exit)");

        match get_input_text("Введите команду").split_whitespace().collect::<Vec<&str>>().as_slice() {
            ["connect", address] => {
                if let Some((ip, port_str)) = address.split_once(':') {
                    if let Ok(port) = port_str.parse::<u16>() {
                        server_clone.connect(ip, port);
                    } else {
                        println!("Некорректный порт: {}", port_str);
                    }
                } else {
                    println!("Неверный формат адреса. Используйте: connect <IP>:<port>");
                }
            }
            ["broadcast", message @ ..] if !message.is_empty() => {
                p2p_protocol.lock().unwrap().response_text(message.join(" "));
            }
            ["exit"] => {
                println!("Выход из программы.");
                break;
            }
            _ => println!("Неверная команда."),
        }
    }

    receiver_thread.join().unwrap();
    server_thread.join().unwrap();
}
