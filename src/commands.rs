use std::io;
use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use crate::app_state::AppState;
use crate::blockchain_functions::count_wallet_amount;
use crate::coin::blockchain::transaction::{SerializedTransaction, Transaction};

pub fn get_input_text(info_text: &str) -> String {
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

pub fn handle_user_commands(app_state: Arc<AppState>) {
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
                        app_state.server.connect(ip, port);
                    } else {
                        println!("Некорректный порт: {}", port_str);
                    }
                } else {
                    println!("Неверный формат адреса. Используйте: connect <IP>:<port>");
                }
            }
            ["broadcast", message @ ..] if !message.is_empty() => {
                app_state.p2p_protocol.lock().unwrap().response_text(message.join(" "));
            }
            ["exit"] => {
                println!("Выход из программы.");
                app_state.running.store(false, Ordering::SeqCst);
                break;
            }
            ["blockchain"] => {
                let last_blocks = app_state.blockchain.lock().unwrap().get_last_n_blocks(50);
                for block in last_blocks {
                    println!("{:?}", block.get_hash());
                    println!("{:?}", block);
                }
            }
            ["transaction", message @ ..] if !message.is_empty() => {
                let message = message.join(" ");
                let sender_key = app_state.wallet.get_public_key_string();
                let receiver_key = get_input_text("Укажи получателя");

                let response_transaction =
                    SerializedTransaction::new(
                        sender_key.clone(),
                        receiver_key.clone(),
                        message, 12.0, 1.0,
                    );

                app_state.p2p_protocol.lock().unwrap().response_transaction(response_transaction);
            }
            ["wallet"] => {
                let my_key = app_state.wallet.get_public_key_string();
                let blockchain = app_state.blockchain.lock().unwrap();
                let result = count_wallet_amount(my_key, &*blockchain);
                println!("Счет кошелька: {}", result);
            }
            ["address"] => {
                println!("Public key: {}", app_state.wallet.get_public_key_string());
            }
            _ => println!("Неверная команда."),
        }
    }
}
