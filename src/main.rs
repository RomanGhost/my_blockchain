mod coin;

use std::sync::{Arc, Mutex};
use std::{io, thread};
use std::io::Read;
use coin::server::Server;

fn get_input_text(info_text: String) -> String {
    let mut input = String::new();
    println!("{}", info_text);
    let input = match io::stdin().read_line(&mut input){
        Ok(i) => input.trim().to_string(),
        Err(e) => {
            eprintln!("Error with reading: {}", e);
            return "".to_string();
        }
    };
    input
}

fn main() {
    let server = Arc::new(Mutex::new(Server::new()));
    let server_clone = Arc::clone(&server);

    let server_address: String = get_input_text("Введите адрес сервера (ip:port)".to_string());
    let connect = get_input_text("Выполнить подключение к серверам?[y/n]".to_string());

    let thread = thread::spawn(move || {
        let mut server_lock = match server_clone.lock() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error locking server for creating: {}", e);
                return;
            }
        };
        server_lock.run(server_address);
    });

    // Подключение к пирам
    if connect == "y" {
        let peer_addresses = vec!["localhost:7879", "localhost:7877"];
        for peer in peer_addresses {
            let mut server_lock = match server.try_lock() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error locking server for peer connected: {}", e);
                    return;
                }
            };
            server_lock.connect_to_peer(peer);
        }
    }

    // // Цикл для передачи сообщений
    // loop {
    //     let mut input = "first message".to_string();
    //     if input == "quit" {
    //         break;
    //     }
    //
    //     let server_lock = match server.try_lock() {
    //         Ok(c) => c,
    //         Err(e) => {
    //             eprintln!("Error locking server for broadcasting: {}", e);
    //             return;
    //         }
    //     };
    //     server_lock.broadcast_message(input.clone());
    //     println!("Вы передали: {}", input);
    //     thread::sleep(std::time::Duration::from_millis(5000));
    //
    // }

    thread.join().unwrap();
}
