use std::collections::HashMap;
use std::io::{Error, ErrorKind, Write};
use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::{Duration, Instant};

use log::{debug, info};

use crate::coin::server::pool::peer_connection::PeerConnection;
use crate::coin::server::pool::pool_message::PoolMessage;
use crate::coin::server::protocol::message::r#type::Message;

pub struct ConnectionPool {
    connections: HashMap<SocketAddr, PeerConnection>,
    timeout: Duration,
    // Каналы для коммуникации с потоком пула
    tx: Sender<PoolMessage>,
    rx: Receiver<PoolMessage>,
    // Каналы для коммуникации с peer
    protocol_tx: Sender<Message>,
}

impl ConnectionPool {
    pub fn new(timeout_secs: u64, tx:Sender<PoolMessage>, rx:Receiver<PoolMessage>, protocol_tx:Sender<Message>) -> Self {
        ConnectionPool {
            connections: HashMap::new(),
            timeout: Duration::from_secs(timeout_secs),
            tx,
            rx,
            protocol_tx,
        }
    }

    // Получить клон отправителя сообщений для пула
    pub fn get_sender(&self) -> Sender<PoolMessage> {
        self.tx.clone()
    }

    // Добавление нового соединения в пул
    fn add_connection(&mut self, addr: SocketAddr, stream: Arc<Mutex<TcpStream>>) {
        debug!("Подключен новый пир: {}", addr);

        self.connections.insert(
            addr,
            PeerConnection {
                addr,
                stream,
                last_seen: Instant::now(),
            },
        );
    }

    // Удаление соединения из пула
    fn remove_connection(&mut self, addr: &SocketAddr) {
        if self.connections.remove(addr).is_some() {
            debug!("Удален пир: {}", addr);
        }
    }

    // Получение списка всех адресов пиров
    pub fn get_peer_addresses(&self) -> Vec<SocketAddr> {
        self.connections.keys().cloned().collect()
    }

    // Отправка сообщения конкретному пиру
    fn send_to_peer(&mut self, addr: &SocketAddr, message: &str) -> Result<(), Error> {
        if let Some(peer) = self.connections.get_mut(addr) {
            if let Ok(mut stream) = peer.stream.lock() {
                stream.write_all(message.as_bytes())?;
                peer.last_seen = Instant::now();
                return Ok(());
            }
        }
        Err(Error::new(ErrorKind::NotFound, "Peer not found or mutex poisoned"))
    }

    // Широковещательная отправка всем пирам
    fn broadcast(&mut self, message: &str) {
        let mut failed_peers = Vec::new();

        for (addr, peer) in &mut self.connections {
            if let Ok(mut stream) = peer.stream.lock() {
                if let Err(_) = stream.write_all(message.as_bytes()) {
                    failed_peers.push(*addr);
                } else {
                    peer.last_seen = Instant::now();
                }
            } else {
                failed_peers.push(*addr);
            }
        }

        // Удаляем соединения, которые не смогли отправить сообщение
        for addr in failed_peers {
            self.remove_connection(&addr);
        }
    }

    // Очистка неактивных соединений
    fn cleanup_inactive(&mut self) {
        let inactive_peers: Vec<SocketAddr> = self.connections
            .iter()
            .filter(|(_, peer)| Instant::now().duration_since(peer.last_seen) > self.timeout)
            .map(|(addr, _)| *addr)
            .collect();

        for addr in inactive_peers {
            debug!("Тайм-аут неактивного пира: {}", addr);
            self.remove_connection(&addr);
        }
    }

    // Основной цикл обработки сообщений пула
    pub fn run(&mut self) {
        loop {
            // Обрабатываем входящие сообщения для пула
            match self.rx.recv_timeout(Duration::from_secs(1)) {
                Ok(PoolMessage::NewPeer(addr, stream)) => {
                    self.add_connection(addr, stream);
                },
                Ok(PoolMessage::PeerDisconnected(addr)) => {
                    self.remove_connection(&addr);
                },
                Ok(PoolMessage::BroadcastMessage(message)) => {
                    debug!("Широковещательное сообщение: {}", message);
                    self.broadcast(&message);
                },
                Ok(PoolMessage::GetPeers(response_tx)) => {
                    let peers = self.get_peer_addresses();
                    let _ = response_tx.send(peers); // Игнорируем ошибку, если получатель отключился
                },
                Ok(PoolMessage::PeerMessage(addr, message)) => {
                    // Обрабатываем сообщение от пира
                    self.protocol_tx.send(Message::RawMessage(message)).unwrap();
                },
                Err(_) => {
                    // Таймаут - чистим неактивные соединения
                    self.cleanup_inactive();
                }
            }
        }
    }
}