mod coin;
use std::io::{self, Write};
use std::sync::{Arc, Condvar, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Duration;
use crate::coin::blockchain::blockchain::Blockchain;
use crate::coin::message::r#type::Message;
use crate::coin::peers::P2PProtocol;
use crate::coin::server::Server;

// Function to get user input with a prompt message.
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

fn server_thread(server_address: String) -> (Server, Receiver<Message>, JoinHandle<()>) {
    // Initialize server.
    let (mut server, rx_server) = Server::new();
    let server_clone = server.clone();

    // Input server address and run it in a separate thread.
    let server_thread = {
        let server_address = server_address.clone();
        thread::spawn(move || {
            server.run(&server_address);
        })
    };
    (server_clone, rx_server, server_thread)
}

fn mining_thread(blockchain: Arc<Mutex<Blockchain>>, mining_flag: Arc<(Mutex<bool>, Condvar)>, p2p_protocol: Arc<Mutex<P2PProtocol>>, running: Arc<AtomicBool>) -> JoinHandle<()> {
    let mining_thread = thread::spawn(move || {
        let (lock, cvar) = &*mining_flag;
        loop {
            // Check if the program is running.
            if !running.load(Ordering::SeqCst) {
                break;
            }

            {
                let mut should_stop = lock.lock().unwrap();
                // Wait until mining is allowed to continue.
                while *should_stop {
                    should_stop = cvar.wait(should_stop).unwrap();
                }
            }

            {
                let mut chain = blockchain.lock().unwrap();
                let iteration_result = chain.proof_of_work();

                // If a new block is found, send it to other nodes.
                if iteration_result {
                    if let Ok(last_block) = chain.get_last_block() {
                        p2p_protocol.lock().unwrap().response_block(last_block, false);
                        // println!("Отправлен новый блок");
                    }
                }
            }
            thread::sleep(Duration::from_millis(1));
        }
    });
    mining_thread
}

fn message_thread(blockchain: Arc<Mutex<Blockchain>>, p2p_protocol: Arc<Mutex<P2PProtocol>>, mining_flag: Arc<(Mutex<bool>, Condvar)>, running: Arc<AtomicBool>, rx_server: Receiver<Message>) -> JoinHandle<()> {
    let message_thread = thread::spawn(move || {
        for received in rx_server {
            if !running.load(Ordering::SeqCst) {
                break; // Exit loop if the program is not running.
            }

            match received {
                Message::RequestLastNBlocksMessage(message) => {
                    let n = message.get_n();
                    let blocks = blockchain.lock().unwrap().get_last_n_blocks(n);
                    for block in blocks {
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

                    // Stop mining upon receiving a new block.
                    {
                        let (lock, _) = &*mining_flag;
                        let mut stop_flag = lock.lock().unwrap();
                        *stop_flag = true;
                    }

                    // println!("Mining stopped");
                    // println!("Adding new block");
                    if is_force_block {
                        // println!("\tForce add");
                        chain.add_force_block(block);
                    } else {
                        // println!("\tUsual add");
                        //При получении блока смотрим, идет ли на опережение наш блокчейн
                        let last_block = chain.get_last_block().unwrap();
                        if last_block.get_datetime() > block.get_datetime()
                        {
                            println!("Проверяем блок по времени");
                            let date_time_after = last_block.get_datetime();
                            let missing_blocks = chain.get_blocks_after(date_time_after);
                            for missing_block in missing_blocks {
                                p2p_protocol.lock().unwrap().response_block(missing_block, true);
                            }
                            return;
                        }
                        if last_block.get_id() > block.get_id() {
                            println!("Проверяем блок по id");
                            let delta = last_block.get_id() - block.get_id() + 1;
                            let missing_blocks = chain.get_last_n_blocks(delta);
                            for missing_block in missing_blocks {
                                p2p_protocol.lock().unwrap().response_block(missing_block, true);
                            }
                            return;
                        }
                        // //Если блок прошел предыдущие проверки, то валидируем его в нашей системе
                        // if last_block.get_previous_hash() != block.get_previous_hash(){
                        //     println!("Конфликт блоков!!!!");
                        // }

                        chain.add_block(block);
                    }
                    // println!("Последний блок: {:?}", chain.get_last_block());

                    // Signal to continue mining.
                    {
                        let (lock, cvar) = &*mining_flag;
                        let mut stop_flag = lock.lock().unwrap();
                        *stop_flag = false;
                        cvar.notify_all();
                    }
                    // println!("Mining started");
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
    });
    message_thread
}

fn main() {
    let server_address = get_input_text("Введите адрес сервера (например, 127.0.0.1:7878)");
    let (server_clone, rx_server, server_thread) = server_thread(server_address);
    let p2p_protocol = server_clone.get_peer_protocol();
    thread::sleep(Duration::from_secs(3)); // Delay for server initialization.

    // Create the blockchain and synchronization variables for mining.
    let blockchain = Arc::new(Mutex::new(Blockchain::new()));
    let mining_flag = Arc::new((Mutex::new(false), Condvar::new()));
    let running = Arc::new(AtomicBool::new(true)); // To control the running state.

    if get_input_text("Запустить майнинг[y/n]:") == "y" {
        let blockchain = Arc::clone(&blockchain);
        let mining_flag = Arc::clone(&mining_flag);
        let p2p_protocol = Arc::clone(&p2p_protocol);
        let running = Arc::clone(&running);

        let mining_thread = mining_thread(blockchain, mining_flag, p2p_protocol, running);
    }

    let blockchain_receiver = Arc::clone(&blockchain);
    let p2p_protocol_receiver = Arc::clone(&p2p_protocol);
    let mining_flag_receiver = Arc::clone(&mining_flag);
    let running_receiver = Arc::clone(&running);
    // Thread for handling incoming messages from the server.
    let receiver_thread = message_thread(blockchain_receiver, p2p_protocol_receiver, mining_flag_receiver, running_receiver, rx_server);

    // Main loop for processing user commands.
    loop {
        println!("\nДоступные команды:");
        println!("1. Подключиться к другому серверу (connect <IP>:<port>)");
        println!("2. Вещать сообщение всем пирами (broadcast <сообщение>)");
        println!("3. Выйти (exit)");
        println!("4. Получить блоки (blockchain)");

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
                running.store(false, Ordering::SeqCst); // Set running to false.
                break;
            }
            ["blockchain"] => {
                let last_blocks = blockchain.lock().unwrap().get_last_n_blocks(50);
                for block in last_blocks {
                    println!("{:?}", block.get_hash());
                    println!("{:?}", block);
                }
            }
            _ => println!("Неверная команда."),
        }
    }

    // Signal threads to stop and wait for them to finish.
    running.store(false, Ordering::SeqCst);
    receiver_thread.join().unwrap();
    server_thread.join().unwrap();

    let last_blocks = blockchain.lock().unwrap().get_last_n_blocks(50);
    for block in last_blocks {
        println!("{:?}", block.get_hash());
        println!("{:?}", block);
    }
}
