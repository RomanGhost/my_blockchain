use std::sync::{Arc, atomic::{AtomicBool, Ordering}, Condvar, Mutex};
use crate::app_state::AppState;
use crate::blockchain_functions::initialize_blockchain;
use crate::coin::blockchain::wallet::Wallet;
use crate::commands::{get_input_text, handle_user_commands};
use crate::message_thread::message_thread;
use crate::mining_thread::mining_thread;
use crate::server_thread::server_thread;
use log::{info, warn, error};
use env_logger;

mod coin;
mod server_thread;
mod blockchain_functions;
mod commands;
mod app_state;
mod message_thread;
mod mining_thread;

fn main() {
    // // Инициализируем логгер
    env_logger::init();
    //
    // // Пример логгирования сообщений с разным уровнем
    info!("Program run");
    // warn!("This is a warning.");
    // error!("This is an error message.");

    // Инициализация сервера
    let address = get_input_text("Введите адрес сервера (например, 127.0.0.1:7878)");
    let (server_clone, rx_server, server_thread_handle) = server_thread(address);
    let peer_protocol = server_clone.get_peer_protocol();

    // Инициализация блокчейна и переменных
    let (blockchain, queue) = initialize_blockchain();

    // Загрузка кошелька
    let wallet = Wallet::load_from_file("cache/wallet.json");

    // Создание состояния приложения
    let app_state = AppState {
        server: server_clone,
        p2p_protocol: peer_protocol,
        blockchain: blockchain.clone(),
        wallet,
        queue: queue.clone(),
        running: Arc::new(AtomicBool::new(true)),
        mining_flag: Arc::new((Mutex::new(true), Condvar::new())), // Управление майнингом
    };
    let app_state = Arc::new(app_state);

    // Запуск потока майнинга, если пользователь выбрал эту опцию
    let mining_thread_handle = if get_input_text("Запустить майнинг? [y/n]") == "y" {
        Some(mining_thread(app_state.clone()))
    } else {
        None
    };

    // Запуск потока для обработки входящих сообщений
    let message_thread_handle = message_thread(app_state.clone(), rx_server);

    // Основной цикл: обработка команд пользователя
    handle_user_commands(app_state.clone());

    // Остановка программы: изменение флага и ожидание завершения потоков
    app_state.running.store(false, Ordering::SeqCst);

    // Ожидание завершения потоков
    if let Some(mining_handle) = mining_thread_handle {
        mining_handle.join().unwrap();
    }
    message_thread_handle.join().unwrap();
    server_thread_handle.join().unwrap();
    info!("Program end");
}
