mod coin;

use std::io::{self, Write, BufRead};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use crate::coin::connection::ConnectionPool;
use crate::coin::peers::P2PProtocol;
use crate::coin::server::Server;

/*
fn main() {
    // Инициализация общего пула соединений
    let connection_pool1 = Arc::new(Mutex::new(ConnectionPool::new()));
    let connection_pool2 = Arc::new(Mutex::new(ConnectionPool::new()));

    // Инициализация протокола для обоих серверов
    let p2p_protocol1 = Arc::new(P2PProtocol::new(connection_pool1.clone()));
    let p2p_protocol2 = Arc::new(P2PProtocol::new(connection_pool2.clone()));

    // Инициализация двух серверов
    let server1 = Server::new(connection_pool1.clone(), p2p_protocol1.clone());
    let server2 = Server::new(connection_pool2.clone(), p2p_protocol2.clone());

    // Запуск первого сервера
    thread::spawn(move || {
        server1.listen("127.0.0.1:7878");
    });

    // Небольшая задержка для корректного запуска сервера
    thread::sleep(Duration::from_secs(1));

    // Запуск второго сервера
    thread::spawn(move || {
        server2.listen("127.0.0.1:7879");
    });

    // Подключение второго сервера к первому
    thread::sleep(Duration::from_secs(2));
    p2p_protocol2.connect_to_peer("127.0.0.1", 8888);

    // Подождем, чтобы увидеть взаимодействие
    thread::sleep(Duration::from_secs(10));
}*/

fn get_input_text(info_text: &str) -> String {
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

fn main() {
    // Инициализация общего пула соединений
    let connection_pool = Arc::new(Mutex::new(ConnectionPool::new()));

    // Инициализация протокола для обоих серверов
    let p2p_protocol = Arc::new(P2PProtocol::new(connection_pool.clone()));

    // Инициализация двух серверов
    let server = Server::new(connection_pool.clone(), p2p_protocol.clone());

    let server_address= get_input_text("Введите адресс сервера: ");

    // Запуск первого сервера
    let server_thread = thread::spawn(move || {
        server.listen(server_address.as_ref());
    });

    // Подключение второго сервера к первому
    thread::sleep(Duration::from_secs(1));
    let connected= get_input_text("Выполнить подключение?[y/n]:");
    if connected == "y" {
        p2p_protocol.connect_to_peer("localhost", 7879);
    }

    server_thread.join().unwrap();
}