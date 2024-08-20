use tokio::io::{AsyncReadExt, AsyncWriteExt}; use 
tokio::net::TcpListener;
#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Создаем TCP-сервер, который будет слушать на порту 
    // 8080
    let listener = 
    TcpListener::bind("5.42.101.120:8085").await?; 
    println!("Server is running on 127.0.0.1:8085"); loop 
    {
        // Принимаем входящее соединение
        let (mut socket, _) = listener.accept().await?; 
        tokio::spawn(async move {
            let mut buffer = [0; 1024]; loop {
                // Читаем данные из сокета
                let n = match socket.read(&mut 
                buffer).await {
                    Ok(n) if n == 0 => return, // Клиент закрыл соединение
                    Ok(n) => n, 
                    Err(_) => {
                        eprintln!("Failed to read from 
                        socket"); return;
                    }
                };
                // Отправляем обратно (эхо) те же данные
                if let Err(e) = 
                socket.write_all(&buffer[..n]).await {
                    eprintln!("Failed to write to socket; 
                    err = {:?}", e); return;
                }
            }
        });
    }
}
