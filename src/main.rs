use std::sync::{Arc, Mutex, TryLockError};
use std::{io, thread};
use std::io::Read;
use std::time::Duration;
use std::sync::mpsc::{self, Sender, Receiver};
use crate::coin::server::Server;

mod coin;

fn get_input_text(info_text: String) -> String {
    let mut input = String::new();
    println!("{}", info_text);
    match io::stdin().read_line(&mut input) {
        Ok(_) => input.trim().to_string(),
        Err(e) => {
            eprintln!("Error with reading: {}", e);
            String::new()
        }
    }
}

fn try_lock_with_retry<T>(
    mutex: &Arc<Mutex<T>>,
    retries: usize,
    delay: Duration,
) -> Option<std::sync::MutexGuard<T>> {
    for _ in 0..retries {
        match mutex.try_lock() {
            Ok(guard) => return Some(guard),
            Err(TryLockError::WouldBlock) => {
                thread::sleep(delay); // Ждём перед новой попыткой
            }
            Err(_) => {
                eprintln!("Failed to acquire lock");
                return None;
            }
        }
    }
    None
}

fn main() {
    let server = Arc::new(Mutex::new(Server::new()));
    let server_clone = Arc::clone(&server);
    let server_clone_broadcast = Arc::clone(&server);

    let server_address: String = get_input_text("Введите адрес сервера (ip:port):".to_string());
    let connect = get_input_text("Выполнить подключение к серверам?[y/n]".to_string());

    // Создаем канал для передачи сообщений между потоками
    let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();

    // Поток для запуска сервера
    let server_run_thread = thread::spawn(move || {
        let mut server_lock = server_clone.lock().unwrap();
        println!("Сервер запускается на адресе: {}", server_address);
        server_lock.run(server_address);  // <-- Сервер запускается, освобождаем мьютекс позже
    });

    // Поток для обработки сообщений из канала
    let server_message_thread = thread::spawn(move || {
        loop {
            match rx.recv() {
                Ok(message) => {
                    println!("Сообщение получено в поток: {}", message);

                    // Повторно захватываем блокировку для отправки сообщений
                    let mut server_lock = match server_clone_broadcast.lock() {
                        Ok(c) => c,
                        Err(e) => {
                            eprintln!("Ошибка блокировки сервера для отправки сообщения: {}", e);
                            continue;
                        }
                    };

                    // Рассылаем сообщение клиентам и узлам
                    server_lock.broadcast_message(message);
                }
                Err(_) => {
                    eprintln!("Канал закрыт. Завершение работы потока.");
                    break;
                }
            }
        }
    });

    // Подключение к пирам
    if connect == "y" {
        let peer_addresses = vec!["localhost:7879", "localhost:7877"];
        for peer in peer_addresses {
            let server_lock = try_lock_with_retry(&server, 5, Duration::from_millis(100));
            if let Some(mut server) = server_lock {
                server.connect_to_peer(peer);
            } else {
                eprintln!("Couldn't acquire lock for peer connection: {}", peer);
            }
        }
    }

    // Цикл для ввода сообщений
    loop {
        let input = get_input_text("Введите сообщение (или 'quit' для выхода):".to_string());
        if input == "quit" {
            break;
        }

        // Передаем сообщение в поток обработки через канал
        if let Err(e) = tx.send(input.clone()) {
            eprintln!("Ошибка отправки сообщения через канал: {}", e);
        }

        thread::sleep(Duration::from_millis(100));
    }

    // Закрываем канал, сигнализируя потоку завершить работу
    drop(tx);

    // Ожидаем завершения серверного потока
    server_run_thread.join().unwrap();
    server_message_thread.join().unwrap();
}
