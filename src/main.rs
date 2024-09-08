use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};
use rust_chat_server::ThreadPool;

fn main() {
    let listener = TcpListener::bind("localhost:7878").unwrap(); // Используем локальный хост для тестирования
    let pool = ThreadPool::new(4);

    for stream in listener.incoming().take(2) {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    // Получаем IP-адрес клиента
    let client_addr_ip;
    match stream.peer_addr() {
        Ok(client_addr) => {
            client_addr_ip = client_addr.to_string();
            println!("Запрос от клиента с IP: {}", client_addr);
        }
        Err(e) => {
            println!("Не удалось получить IP-адрес клиента: {}", e);
            return; // Завершаем выполнение функции, если не удалось получить IP
        }
    }

    let buf_reader = BufReader::new(&mut stream);
    let http_request_line = match buf_reader.lines().next() {
        Some(Ok(line)) => line,
        Some(Err(e)) => {
            println!("Ошибка при чтении строки запроса: {}", e);
            return;
        }
        None => {
            println!("Пустой запрос");
            return;
        }
    };

    let (status_line, filename) = match &http_request_line[..]{
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "index.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "index.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
    };
    let filepath = format!("pages/{filename}");
    //     if http_request_line == "GET / HTTP/1.1" {
    //     thread::sleep(Duration::from_secs(5));
    //     ("HTTP/1.1 200 OK", "pages/index.html")
    // } else {
    //     ("HTTP/1.1 404 NOT FOUND", "pages/404.html")
    // };

    let raw_content = fs::read_to_string(filepath).unwrap_or_else(|e| {
        println!("Ошибка при чтении файла: {}", e);
        String::new()
    });
    let content = raw_content.replace(":ip", &client_addr_ip);
    let length = content.len();

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{content}");

    if let Err(e) = stream.write_all(response.as_bytes()) {
        println!("Ошибка при отправке ответа клиенту: {}", e);
    }
}
