use std::collections::BinaryHeap;
use std::io::{self, Write};
use std::sync::{Arc, Condvar, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use rsa::pkcs1::DecodeRsaPublicKey;

use crate::coin::blockchain::blockchain::Blockchain;
use crate::coin::blockchain::transaction::{SerializedTransaction, Transaction};
use crate::coin::blockchain::wallet::Wallet;
use crate::coin::message::r#type::Message;
use crate::coin::peers::P2PProtocol;
use crate::coin::server::Server;

mod coin;

/// TODO Добавить логирование в программу

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

fn count_wallet_amount(my_public_key: String, blockchain: &Blockchain) -> f64 {
    let chain = &blockchain.chain;
    let mut amount = 0.0;
    for block in chain {
        let transactions = block.get_transactions();
        for transaction in transactions {
            if transaction.get_receiver() == my_public_key {
                amount += transaction.transfer;
            }
            if transaction.get_sender() == my_public_key {
                amount -= transaction.transfer;
            }
        }
    }

    amount
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

fn mining_thread(blockchain: Arc<Mutex<Blockchain>>, mining_flag: Arc<(Mutex<bool>, Condvar)>, p2p_protocol: Arc<Mutex<P2PProtocol>>, running: Arc<AtomicBool>, queue: Arc<Mutex<BinaryHeap<SerializedTransaction>>>) -> JoinHandle<()> {
    let mining_thread = thread::spawn(move || {
        let mut transactions: Vec<SerializedTransaction> = vec![];
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
                let mut lock_queue = queue.lock().unwrap();

                if lock_queue.len() > 0 && transactions.len() < 4 {
                    let num_iteration = 4 - transactions.len();
                    for _ in 0..num_iteration {
                        let transaction = lock_queue.pop();
                        // dbg!(&transaction);
                        match transaction {
                            Some(t) => transactions.push(t),
                            None => println!("Нет доступных транзакций для обработки."),
                        }
                    }
                    chain.clear_nonce();
                }

                let iteration_result = chain.proof_of_work(transactions.clone());

                // If a new block is found, send it to other nodes.
                if iteration_result {
                    if let Ok(last_block) = chain.get_last_block() {
                        p2p_protocol.lock().unwrap().response_block(last_block, false);
                        transactions.clear();
                        println!("Отправлен новый блок");
                    }
                }
            }
            thread::sleep(Duration::from_millis(1));
        }
    });
    mining_thread
}

fn message_thread(blockchain: Arc<Mutex<Blockchain>>, p2p_protocol: Arc<Mutex<P2PProtocol>>, mining_flag: Arc<(Mutex<bool>, Condvar)>, running: Arc<AtomicBool>, rx_server: Receiver<Message>, queue: Arc<Mutex<BinaryHeap<SerializedTransaction>>>) -> JoinHandle<()> {
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
                    let new_block = message.get_block();
                    println!("Получен новый блок: {}", new_block.get_id());

                    let mut chain = blockchain.lock().unwrap();

                    {
                        let (lock, _) = &*mining_flag;
                        let mut stop_flag = lock.lock().unwrap();
                        *stop_flag = true;
                    }


                    if is_force_block {
                        chain.add_force_block(new_block);
                    } else {
                        chain.add_block(new_block);
                    }


                    // Signal to continue mining.
                    {
                        let (lock, cvar) = &*mining_flag;
                        let mut stop_flag = lock.lock().unwrap();
                        *stop_flag = false;
                        cvar.notify_all();
                    }
                    // println!("Mining started");
                }
                Message::ResponseTransactionMessage(message) => {
                    let new_transaction = message.get_transaction();
                    println!("Получена новая транзакция! > {:?}", new_transaction);
                    queue.lock().unwrap().push(new_transaction);
                    println!("Транзакция добавлена в очередь");
                }
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
    // TODO Переделать функцию на новый лад
    let server_address = get_input_text("Введите адрес сервера (например, 127.0.0.1:7878)");
    let (server_clone, rx_server, server_thread) = server_thread(server_address);
    let p2p_protocol = server_clone.get_peer_protocol();
    thread::sleep(Duration::from_secs(3)); // Delay for server initialization.

    //Create the priority queue
    let queue = Arc::new(Mutex::new(BinaryHeap::new()));
    let queue4blockchain = Arc::clone(&queue);
    let queue4message = Arc::clone(&queue);

    // Create the blockchain and synchronization variables for mining.
    let blockchain = Arc::new(Mutex::new(Blockchain::new()));
    let mining_flag = Arc::new((Mutex::new(false), Condvar::new()));
    let running = Arc::new(AtomicBool::new(true)); // To control the running state.

    // Wallet load info
    let wallet = Wallet::load_from_file("cache/wallet.json");
    let public_key_string = wallet.get_public_key_string();
    println!("Public wallet key: {}", public_key_string);


    if get_input_text("Запустить майнинг[y/n]:") == "y" {
        let blockchain = Arc::clone(&blockchain);
        let mining_flag = Arc::clone(&mining_flag);
        let p2p_protocol = Arc::clone(&p2p_protocol);
        let running = Arc::clone(&running);

        mining_thread(blockchain, mining_flag, p2p_protocol, running, queue4blockchain);
    }

    let blockchain_receiver = Arc::clone(&blockchain);
    let p2p_protocol_receiver = Arc::clone(&p2p_protocol);
    let mining_flag_receiver = Arc::clone(&mining_flag);
    let running_receiver = Arc::clone(&running);
    // Thread for handling incoming messages from the server.
    let receiver_thread = message_thread(blockchain_receiver, p2p_protocol_receiver, mining_flag_receiver, running_receiver, rx_server, queue4message);

    // Main loop for processing user commands.
    loop {
        println!("\nДоступные команды:");
        println!("1. Подключиться к другому серверу (connect <IP>:<port>)");
        println!("2. Вещать сообщение всем пирами (broadcast <сообщение>)");
        println!("3. Выйти (exit)");
        println!("4. Получить блоки (blockchain)");
        println!("5. Создать транзакцию (transaction)");
        println!("6. Получить публичный ключ (address)");
        println!("7. Получить счет кошелька (wallet)");

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
            ["transaction", message @ ..] if !message.is_empty() => {
                let message = message.join(" ");
                let sender_key = wallet.get_public_key_string();
                let receiver_key = get_input_text("Укажи получателя");

                let mut response_transaction =
                    SerializedTransaction::new(
                        sender_key.clone(),
                        receiver_key.clone(),
                        message, 12.0, 1.0,
                    );

                let mut signed_transaction = response_transaction.clone();
                let transaction = Transaction::deserialize(response_transaction);

                match transaction {
                    Ok(mut transaction) => {
                        transaction.sign(wallet.get_private_key());
                        signed_transaction = transaction.serialize();
                    }
                    Err(e) => {
                        eprintln!("{}", e);
                    }
                }
                println!("Подпись создана");
                p2p_protocol.lock().unwrap().response_transaction(signed_transaction);
            }
            ["wallet"] => {
                let my_key = wallet.get_public_key_string();
                let blockchain = blockchain.lock().unwrap();
                let result = count_wallet_amount(my_key, &*blockchain);
                println!("Счет кошелька: {}", result);
            }
            ["address"] => {
                println!("Public key: {}", public_key_string);
            }
            _ => println!("Неверная команда."),
        }
    }
    /// TODO проработать правильный выход из программы

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
