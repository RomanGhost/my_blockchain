use std::collections::HashSet; 
use std::sync::{Arc, Mutex}; 
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader}; 
use tokio::net::TcpListener; 
use tokio::sync::broadcast;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Запуск TCP-сервера на 127.0.0.1:8080
    let url = "5.42.101.120:8085";
    let listener =  TcpListener::bind(url).await?;
    
    // Создаем канал для широковещательной рассылки 
    // сообщений
    let (tx, _rx) = broadcast::channel(10);
    
    // Создаем потокобезопасную общую структуру для 
    // хранения клиентов
    let clients = Arc::new(Mutex::new(HashSet::new())); 
    println!("Chat server running on ${url}"); 
    loop {
        // Ожидание нового подключения
        let (socket, _) = listener.accept().await?; 
        let tx = tx.clone(); 
        let mut rx = tx.subscribe(); 
        let clients = Arc::clone(&clients); 
        tokio::spawn(async move {
            let (reader, mut writer) = socket.into_split(); 
            let mut reader = BufReader::new(reader); 
            let mut nickname = String::new();
            // Запрос ника у пользователя
            writer.write_all(b"Enter your nickname: ").await.unwrap(); 
            reader.read_line(&mut nickname).await.unwrap(); 
            let nickname = nickname.trim().to_string();
            // Сообщаем всем о новом подключении
            tx.send(format!("{} has joined the chat", nickname)).unwrap(); 
            {
                let mut clients = clients.lock().unwrap(); 
                clients.insert(nickname.clone());
            }
            loop { 
                let mut message = String::new(); 
                tokio::select! {
                    // Чтение сообщения от клиента
                    result = reader.read_line(&mut message) => {
                        if result.unwrap() == 0 { 
                            break;
                        }
                        let message = message.trim().to_string(); 
                        tx.send(format!("{}: {}", nickname, message)).unwrap();
                    }
                    // Получение сообщения из канала и 
                    // отправка клиенту
                    result = rx.recv() => { 
                        let message = result.unwrap(); 
                        writer.write_all(message.as_bytes()).await.unwrap(); 
                        writer.write_all(b"\n").await.unwrap();
                    }
                }
            }
            // Сообщаем всем о выходе клиента
            tx.send(format!("{} has left the chat", nickname)).unwrap(); 
            let mut clients = clients.lock().unwrap(); 
            clients.remove(&nickname);
        });
    }
}
