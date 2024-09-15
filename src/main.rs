mod connection;
mod server;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use server::Server; // Импортируем структуру Server из модуля server

#[tokio::main]
async fn main() {
    // Создание экземпляра сервера с заданным IP и портом
    let server = Server::new("0.0.0.0", 8888);
    server.run().await; // Запуск сервера
}
