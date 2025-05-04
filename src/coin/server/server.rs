use std::io::{Error, ErrorKind, Read};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

use log::{debug, error, info, warn};

use crate::coin::server::pool::pool_message::PoolMessage;

pub struct Server {
    pool_tx: Sender<PoolMessage>
}

impl Server{
    pub fn new(pool_tx: Sender<PoolMessage>) -> Self{

        Server{pool_tx}
    }

    pub fn run(&mut self, address: &str) -> Result<(), Error>{
        let listener = TcpListener::bind(address)?;
        info!("P2P сервер запущен на {}", address);

        for stream in listener.incoming(){
            match stream {
                Ok(stream) => {
                    if stream.local_addr()? == stream.peer_addr()?{
                        stream.shutdown(Shutdown::Read).expect("Error close input connection");
                    }

                    let stream = stream;
                    let pool_tx = self.pool_tx.clone();

                    // Запускаем отдельный поток для каждого пира
                    thread::spawn(move || {
                        handle(stream, pool_tx);
                    });
                },
                Err(e) => {
                    error!("Ошибка при принятии соединения: {}", e);
                }
            }
        }
        Ok(())
    }

    pub fn connect(&self, address:String) -> Result<(), Error>{
        let stream = TcpStream::connect(address.clone());
        match stream {
            Ok(stream) => {
                if stream.local_addr() ? == stream.peer_addr() ? {
                    stream.shutdown(Shutdown::Read).expect("Is same addresses");
                }

                let stream = stream;
                let pool_tx = self.pool_tx.clone();

                // Запускаем отдельный поток для каждого пира
                thread::spawn(move || {
                    handle(stream, pool_tx);
                });
            },
            Err(e) => {
                error!("Error connect to {}, err:{}",address, e);
            }
        }
        Ok(())
    }

    pub fn get_pool_sender(&self) -> Sender<PoolMessage>{
        self.pool_tx.clone()
    }
}

fn handle(
    stream: TcpStream,
    pool_tx: Sender<PoolMessage>
) -> Result<(), Error>{
    let addr = stream.peer_addr()?;
    info!("Запущен поток для пира {}", addr);

    let stream = Arc::new(Mutex::new(stream));
    // Уведомляем пул о новом пире
    let _ = pool_tx.send(PoolMessage::NewPeer(addr, stream.clone()));

    let mut buffer = [0; 1024];
    let stream_clone = stream.clone();

    // Установим таймаут для чтения
    if let Ok(locked_stream) = stream_clone.lock() {
        let _ = locked_stream.set_read_timeout(Some(Duration::from_millis(500)));
    }

    loop {
        // Блокируемся и ждем данных от пира
        let read_result = {
            if let Ok(mut locked_stream) = stream.lock() {
                locked_stream.read(&mut buffer)
            } else {
                // Мьютекс захвачен другим потоком и вызвал панику
                break;
            }
        };

        match read_result {
            Ok(0) => {
                // Соединение закрыто
                warn!("Пир {} отключился", addr);
                break;
            },
            Ok(n) => {
                // Получены данные
                if let Ok(message) = std::str::from_utf8(&buffer[0..n]) {
                    debug!("message get to server: {:?}", message);
                    let _ = pool_tx.send(PoolMessage::PeerMessage(addr, message.to_string()));
                }
            },
            Err(ref e) if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut => {
                // Таймаут чтения, продолжаем
                thread::sleep(Duration::from_millis(100));
                continue;
            },
            Err(_) => {
                // Ошибка чтения
                error!("Ошибка чтения от пира {}", addr);
                break;
            }
        }
    }

    // Уведомляем пул об отключении пира
    let _ = pool_tx.send(PoolMessage::PeerDisconnected(addr));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coin::server::pool::pool_message::PoolMessage;
    use std::io::{Read, Write};
    use std::net::{Shutdown, TcpListener, TcpStream};
    use std::sync::mpsc::channel;
    use std::thread;
    use std::time::Duration;

    /// Проверяем чистый handle: NewPeer → PeerMessage → PeerDisconnected
    #[test]
    fn test_handle_lifecycle() {
        let (tx_pool, rx_pool) = channel();

        // Листенер для имитации сервера
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind failed");
        let addr = listener.local_addr().unwrap();

        // В фоне принимаем соединение и запускаем handle
        let tx2 = tx_pool.clone();
        thread::spawn(move || {
            let (stream, _) = listener.accept().expect("accept failed");
            handle(stream, tx2).expect("handle failed");
        });

        // Клиент подключается
        let mut client = TcpStream::connect(addr).expect("connect failed");

        // 1) Должен прийти NewPeer(addr, _)
        match rx_pool.recv_timeout(Duration::from_secs(1)).expect("no NewPeer") {
            PoolMessage::NewPeer(a, _) => assert_eq!(a.ip(), addr.ip()),
            other => panic!("expected NewPeer, got {:?}", other),
        }

        // 2) Посылаем текст и ждём PeerMessage
        let msg = "hello\n";
        client.write_all(msg.as_bytes()).unwrap();
        match rx_pool.recv_timeout(Duration::from_secs(1)).expect("no PeerMessage") {
            PoolMessage::PeerMessage(a, data) => {
                assert_eq!(a.ip(), addr.ip());
                assert_eq!(data, msg);
            }
            other => panic!("expected PeerMessage, got {:?}", other),
        }

        // 3) Закрываем клиент
        client.shutdown(Shutdown::Both).unwrap();
        match rx_pool.recv_timeout(Duration::from_secs(1)).expect("no PeerDisconnected") {
            PoolMessage::PeerDisconnected(a) => assert_eq!(a.ip(), addr.ip()),
            other => panic!("expected PeerDisconnected, got {:?}", other),
        }
    }

    /// Проверяем метод connect (внутри вызывает handle) и get_pool_sender
    #[test]
    fn test_server_connect_and_sender() {
        let (tx_pool, rx_pool) = channel();
        let server = Server::new(tx_pool.clone());

        // get_pool_sender даёт рабочий клон
        let tx2 = server.get_pool_sender();
        tx2.send(PoolMessage::BroadcastMessage("ping".into())).unwrap();
        match rx_pool.recv_timeout(Duration::from_secs(1)).unwrap() {
            PoolMessage::BroadcastMessage(s) => assert_eq!(s, "ping"),
            other => panic!("expected BroadcastMessage, got {:?}", other),
        }

        // Теперь проверяем connect()
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        // Приём + handle через connect
        let tx3 = tx_pool.clone();
        thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            handle(stream, tx3).unwrap();
        });

        server.connect(addr.to_string()).unwrap();

        // Должен прийти NewPeer
        match rx_pool.recv_timeout(Duration::from_secs(1)).unwrap() {
            PoolMessage::NewPeer(a, _) => assert_eq!(a.ip(), addr.ip()),
            other => panic!("expected NewPeer, got {:?}", other),
        }
    }

    /// Проверяем метод run: запускаем сервер, подключаемся, шлём данные, ожидаем все три этапа
    #[test]
    fn test_server_run() {
        let (tx_pool, rx_pool) = channel();
        let mut server = Server::new(tx_pool.clone());

        // Найдём свободный порт, чтобы передать его в run()
        let temp_listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = temp_listener.local_addr().unwrap().port();
        drop(temp_listener);

        // Запускаем run в фоне
        thread::spawn(move || {
            server.run(&format!("127.0.0.1:{}", port)).expect("run failed");
        });

        // Немного ждём, чтобы сервер успел встать на прослушку
        thread::sleep(Duration::from_millis(100));

        // Подключаемся как клиент
        let mut client = TcpStream::connect(("127.0.0.1", port)).expect("client connect failed");

        // 1) NewPeer
        match rx_pool.recv_timeout(Duration::from_secs(1)).expect("no NewPeer") {
            PoolMessage::NewPeer(a, _) => assert_eq!(a.ip().to_string(), "127.0.0.1"),
            other => panic!("expected NewPeer, got {:?}", other),
        }

        // 2) Отправляем сообщение, ждём PeerMessage
        let test_msg = "msg over run\n";
        client.write_all(test_msg.as_bytes()).unwrap();
        match rx_pool.recv_timeout(Duration::from_secs(1)).expect("no PeerMessage") {
            PoolMessage::PeerMessage(_, data) => assert_eq!(data, test_msg),
            other => panic!("expected PeerMessage, got {:?}", other),
        }

        // 3) Закрываем клиент, ждём PeerDisconnected
        client.shutdown(Shutdown::Both).unwrap();
        match rx_pool.recv_timeout(Duration::from_secs(2)).expect("no PeerDisconnected") {
            PoolMessage::PeerDisconnected(_) => {}
            other => panic!("expected PeerDisconnected, got {:?}", other),
        }
    }
}
