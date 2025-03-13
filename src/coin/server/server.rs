use std::fmt::format;
use std::format;
use std::io::{Error, ErrorKind, Read};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::time::Duration;

use log::{error, info, warn};

use crate::coin::app_state::AppState;
use crate::coin::node::blockchain::block::Block;
use crate::coin::server::pool::connection_pool::ConnectionPool;
use crate::coin::server::pool::pool_message::PoolMessage;
use crate::coin::server::protocol::message::r#type::Message;
use crate::coin::server::protocol::p2p_protocol::P2PProtocol;

pub struct Server {
    pool_tx: Sender<PoolMessage>
}

impl Server{
    pub fn new(pool_tx: Sender<PoolMessage>) -> Self{

        Server{ pool_tx}
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

    pub fn connect(&self, address:&str, port:u16) -> Result<(), Error>{
        let stream = TcpStream::connect(format!("{}:{}", address, port));
        match stream {
            Ok(stream) => {
                if stream.local_addr() ? == stream.peer_addr() ? {
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
                error!("Error connect to {}:{}, err:{}",address, port, e);
            }
        }

        Ok(())
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
    if let Ok(mut locked_stream) = stream_clone.lock() {
        let _ = locked_stream.set_read_timeout(Some(Duration::from_secs(1)));
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