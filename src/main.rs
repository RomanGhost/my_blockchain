mod coin;

use std::sync::{Arc, Mutex};
use std::{io, thread};
use coin::server::Server;

fn get_input_text(info_text:&str) -> String {
    let mut input = String::new();
    println!("{}", info_text);
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn main() {
    let server = Arc::new(Mutex::new(Server::new()));

    let server_clone = Arc::clone(&server); // Клонируем для передачи в поток

    let mut server_address: String = "localhost:7878".to_string(); //String::new();

    let thread;
    {
        thread = thread::spawn(move || {
            let mut server = server_clone.try_lock().unwrap();
            server.run(server_address.clone());
        });
    }

    let connect = "y";//get_input_text("Выполнить подключение к серверам?[y/n]");
    if connect == "y" {
        //чтобы ограничить время жизни блокировки
        {
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
    }
    // Теперь можно передавать сообщения, используя блокировку
    loop {
        //get_input_text("Фраза для передачи:");
        //"hello".to_string();
        let mut input = "hello".to_string();
        if input == "quit" {
            break;
        }

        let server_lock = match server.try_lock() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error locking server for broadcasting: {}", e);
                return;
            }
        };
        server_lock.broadcast_message(input.clone());
        println!("Вы передали: {}", input);
        thread::sleep(std::time::Duration::from_secs(5));
    }
    thread.join().unwrap();
}