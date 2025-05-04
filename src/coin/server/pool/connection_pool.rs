use std::collections::HashMap;
use std::io::{Error, ErrorKind, Write};
use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, Sender};
use std::time::{Duration, Instant};

use log::{debug, warn};

use crate::coin::server::pool::peer_connection::PeerConnection;
use crate::coin::server::pool::pool_message::PoolMessage;
use crate::coin::server::protocol::message::r#type::Message;
use crate::coin::server::protocol::message::r#type::Message::RequestMessageInfo;
use crate::coin::server::protocol::message::request::MessageFirstInfo;
use crate::coin::server::protocol::message::response::PeerMessage;

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
        self.connections.insert(
            addr,
            PeerConnection {
                addr,
                stream,
                last_seen: Instant::now(),
                buffer: String::new(),
            },
        );
        debug!("Подключен новый пир: {} len: {}", addr, self.connections.len());

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
                stream.write_all(format!("{}\n", message).as_bytes())?;
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
                if let Err(_) = stream.write_all(format!("{}\n", message).as_bytes()) {
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
            match self.rx.recv_timeout(Duration::from_secs(600)) {
                Ok(PoolMessage::NewPeer(addr, stream)) => {
                    // debug!("connected new peer, now peers: {}", self.connections.len());
                    self.add_connection(addr, stream);
                    self.protocol_tx.send(Message::ResponsePeerMessage(PeerMessage::new(addr.ip().to_string()))).unwrap();
                    self.protocol_tx.send(RequestMessageInfo(MessageFirstInfo::new())).unwrap();
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
                    response_tx.send(peers).expect("TODO: panic message"); // Игнорируем ошибку, если получатель отключился
                },
                Ok(PoolMessage::PeerMessage(addr, message)) => {
                    self.handle_peer_message(addr, message);
                },
                Err(err) => {
                    warn!("Error timeout pool: {}", err);
                    // Таймаут - чистим неактивные соединения
                    self.cleanup_inactive();
                }
            }
        }
    }

    // Новая функция для обработки входящих сообщений
    fn handle_peer_message(&mut self, addr: SocketAddr, message: String) {
        // Сначала обрабатываем буфер и извлекаем сообщения
        let messages = if let Some(peer) = self.connections.get_mut(&addr) {
            let mut buffer = std::mem::take(&mut peer.buffer);
            buffer.push_str(&message);

            let mut messages = Vec::new();
            while let Some(pos) = buffer.find('\n') {
                let (message, remaining) = buffer.split_at(pos);
                messages.push(message.to_string());
                buffer = remaining[1..].to_string();
            }
            peer.buffer = buffer;
            Some(messages)
        } else {
            debug!("Получено сообщение от неизвестного пира: {}", addr);
            None
        };

        // Теперь обрабатываем сообщения без одновременного заимствования connections
        if let Some(messages) = messages {
            for message in messages {
                // self.broadcast(&message);
                self.protocol_tx.send(Message::RawMessage(message))
                    .unwrap_or_else(|_| debug!("Ошибка отправки"));

                // Обновляем время активности после broadcast
                if let Some(peer) = self.connections.get_mut(&addr) {
                    peer.last_seen = Instant::now();
                }
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use std::io::Read;
    use super::*;
    use std::net::{TcpListener, TcpStream, SocketAddr};
    use std::sync::{Arc, Mutex, mpsc};
    use std::thread;
    use std::time::{Duration, Instant};

    /// Хелпер для создания пары подключённого TcpStream и регистрации
    /// серверной стороны в пуле.
    fn setup_connection(pool: &mut ConnectionPool, addr: &mut SocketAddr) -> TcpStream {
        // Листенер на случайном порту
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind failed");
        *addr = listener.local_addr().unwrap();

        // Стрим клиента
        let client = TcpStream::connect(*addr).expect("connect failed");
        // Принимаем на стороне сервера
        let (server, _) = listener.accept().expect("accept failed");

        // Регистрируем в пуле
        pool.add_connection(*addr, Arc::new(Mutex::new(server)));
        client
    }

    #[test]
    fn test_add_connection_and_get_peer_addresses() {
        let (tx_pool, rx_pool) = mpsc::channel();
        let (tx_proto, _rx_proto) = mpsc::channel();
        let mut pool = ConnectionPool::new(10, tx_pool, rx_pool, tx_proto);

        let mut addr = "127.0.0.1:0".parse().unwrap();
        let _client = setup_connection(&mut pool, &mut addr);

        let peers = pool.get_peer_addresses();
        assert_eq!(peers, vec![addr]);
    }

    #[test]
    fn test_send_to_peer() {
        use std::io::{BufRead, BufReader};

        let (tx_pool, rx_pool) = mpsc::channel();
        let (tx_proto, _rx_proto) = mpsc::channel();
        let mut pool = ConnectionPool::new(10, tx_pool, rx_pool, tx_proto);

        // Настраиваем соединение
        let mut addr = "127.0.0.1:0".parse().unwrap();
        let client = setup_connection(&mut pool, &mut addr);

        // Отправляем сообщение
        pool.send_to_peer(&addr, "hello").expect("send_to_peer failed");

        // Читаем ровно одну строку (до '\n')
        let mut reader = BufReader::new(client);
        let mut line = String::new();
        reader.read_line(&mut line).expect("read_line failed");

        assert_eq!(line, "hello\n");
    }


    #[test]
    fn test_broadcast_and_remove_failed_peer() {
        use std::thread;

        let (tx_pool, rx_pool) = mpsc::channel();
        let (tx_proto, _rx_proto) = mpsc::channel();
        let mut pool = ConnectionPool::new(10, tx_pool, rx_pool, tx_proto);

        // 1) Настраиваем рабочее соединение
        let mut addr1 = "127.0.0.1:0".parse().unwrap();
        let _good_client = setup_connection(&mut pool, &mut addr1);

        // 2) Настраиваем «сломанное» соединение через Poisoned Mutex:
        //    - создаём listener
        //    - коннектимся клиентом/сервером
        //    - после accept() получаем серверный stream
        //    - оборачиваем его в Arc<Mutex<_>>
        //    - сразу же в отдельном потоке паника при захвате lock, вызывая PoisonError
        let listener2 = TcpListener::bind("127.0.0.1:0").unwrap();
        let bad_addr = listener2.local_addr().unwrap();
        let _bad_client = TcpStream::connect(bad_addr).unwrap();
        let (bad_server, _) = listener2.accept().unwrap();

        let m = Arc::new(Mutex::new(bad_server));
        // Poison the mutex
        {
            let m_clone = Arc::clone(&m);
            let _ = thread::spawn(move || {
                let _guard = m_clone.lock().unwrap();
                panic!("poisoning mutex");
            }).join();
        }
        // Теперь Mutex в состоянии PoisonError
        pool.add_connection(bad_addr, m);

        // 3) Широковещаем — для "сломанного" peers lock() даст Err, и он удалится
        pool.broadcast("ping");

        // 4) Остаётся только рабочий addr1
        let peers = pool.get_peer_addresses();
        assert_eq!(peers, vec![addr1]);
    }



    #[test]
    fn test_cleanup_inactive() {
        // timeout = 0, чтобы сразу считать всех неактивными
        let (tx_pool, rx_pool) = mpsc::channel();
        let (tx_proto, _rx_proto) = mpsc::channel();
        let mut pool = ConnectionPool::new(0, tx_pool, rx_pool, tx_proto);

        let mut addr = "127.0.0.1:0".parse().unwrap();
        let _client = setup_connection(&mut pool, &mut addr);

        // сразу удаляем «неактивных»
        pool.cleanup_inactive();
        assert!(pool.get_peer_addresses().is_empty());
    }

    #[test]
    fn test_handle_peer_message_and_protocol_forwarding() {
        let (tx_pool, rx_pool) = mpsc::channel();
        let (tx_proto, rx_proto) = mpsc::channel();
        let mut pool = ConnectionPool::new(10, tx_pool, rx_pool, tx_proto);

        let mut addr = "127.0.0.1:0".parse().unwrap();
        let _client = setup_connection(&mut pool, &mut addr);

        // отправляем часть и полный, проверяем делим на сообщения
        pool.handle_peer_message(addr, "msg1\nmsg2\npartial".to_string());
        // теперь буфер для addr содержит "partial"
        pool.handle_peer_message(addr, "1\n".to_string());

        // Должны получить 3 RawMessage: "msg1", "msg2", "partial1"
        let mut collected = Vec::new();
        for _ in 0..3 {
            if let Ok(Message::RawMessage(m)) = rx_proto.recv_timeout(Duration::from_secs(1)) {
                collected.push(m);
            }
        }
        assert_eq!(collected, vec!["msg1", "msg2", "partial1"]);
    }
}
