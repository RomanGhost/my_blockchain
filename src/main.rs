use std::sync::{Arc, Mutex, TryLockError};
use std::{io, thread};
use std::io::Read;
use std::time::Duration;
use std::sync::mpsc::{self, Sender, Receiver};

mod coin;
use coin::server::Server;

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
    let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();

    let server_clone = Arc::clone(&server);
    let server_address = get_input_text("Введите адрес сервера (ip:port):".to_string());

    // Поток для обработки входящих соединений и клиентов
    let server_thread = thread::spawn(move || {
        let mut server_lock = server_clone.lock().expect("Ошибка блокировки сервера");
        server_lock.run(server_address);
    });
    {
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

    // Поток для обработки ввода
    let input_thread = thread::spawn(move || {
        loop {
            let input = get_input_text("Text: ".to_string());
            if let Err(_) = tx.send(input) {
                eprintln!("Ошибка при отправке сообщения");
                break;
            }
        }
    });

    // Основной цикл обработки сообщений
    loop {
        match rx.recv() {
            Ok(message) => {
                let server_lock = server.lock().expect("Ошибка блокировки сервера");
                server_lock.broadcast_message(message);
            }
            Err(e) => {
                eprintln!("Ошибка получения сообщения из канала: {}", e);
                break;
            }
        }
    }

    // Ожидаем завершения потоков
    server_thread.join().expect("Ошибка при завершении серверного потока");
    input_thread.join().expect("Ошибка при завершении потока ввода");
}