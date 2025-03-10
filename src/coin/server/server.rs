use std::io::{self, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::{mpsc, Arc, Mutex};
use std::sync::mpsc::Sender;
use std::thread;
use std::time::{Duration, Instant};

use log::{debug, error, info, warn};
use crate::coin::server::connection::ConnectionPool;
use crate::coin::server::protocol::message::r#type::Message;
use crate::coin::server::protocol::peers::P2PProtocol;
use crate::coin::server::errors::ServerError;

const HANDSHAKE_MESSAGE: &str = "NEW_CONNECT!\r\n";
const TIMEOUT_SECONDS: u64 = 600;
const BUFFER_SIZE: usize = 4096;
const CONNECTION_TIMEOUT: u64 = 10; // 10 секунд для установки соединения

#[derive(Clone)]
pub struct Server {
    connection_pool: Arc<Mutex<ConnectionPool>>,
    p2p_protocol: Arc<Mutex<P2PProtocol>>,
}

impl Server {
    pub fn new(tx: Sender<Message>) -> Self {
        let connection_pool = Arc::new(Mutex::new(ConnectionPool::new(BUFFER_SIZE)));
        let p2p_protocol = Arc::new(Mutex::new(P2PProtocol::new(connection_pool.clone(), tx)));

        Server {
            connection_pool,
            p2p_protocol,
        }
    }

    pub fn run(&mut self, address: &str) -> io::Result<()> {
        let listener = TcpListener::bind(address)?;
        info!("Сервер запущен и слушает адрес {}", address);

        // Запуск отдельного потока для периодической очистки неактивных соединений
        // let connection_pool_clone = self.connection_pool.clone();
        // thread::spawn(move || {
        //     loop {
        //         thread::sleep(Duration::from_secs(60)); // Проверка каждую минуту
        //         connection_pool_clone.lock().unwrap().prune_inactive_connections(TIMEOUT_SECONDS);
        //     }
        // });

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    // Устанавливаем таймаут на новое соединение
                    stream.set_read_timeout(Some(Duration::from_secs(CONNECTION_TIMEOUT)))?;
                    stream.set_write_timeout(Some(Duration::from_secs(CONNECTION_TIMEOUT)))?;

                    let connection_pool = self.connection_pool.clone();
                    let p2p_protocol = self.p2p_protocol.clone();

                    match stream.peer_addr() {
                        Ok(peer_addr) => {
                            let peer_address = peer_addr.ip().to_string();

                            // Проверяем, есть ли уже соединение с этим адресом
                            if connection_pool.lock().unwrap().connection_exist(&peer_address) {
                                info!("Уже существует соединение с {}, закрываем дубликат", peer_address);
                                let _ = stream.shutdown(Shutdown::Both);
                                continue;
                            }

                            thread::spawn(move || {
                                if let Err(e) = handle_connection(&peer_address, &mut stream, &connection_pool, &p2p_protocol, false) {
                                    warn!("Ошибка обработки соединения: {:?}", e);
                                    let _ = stream.shutdown(Shutdown::Both);
                                }
                            });
                        }
                        Err(e) => {
                            warn!("Не удалось получить адрес пира: {:?}", e);
                            let _ = stream.shutdown(Shutdown::Both);
                        }
                    }
                }
                Err(e) => {
                    warn!("Ошибка принятия соединения: {:?}", e);
                }
            }
        }

        Ok(())
    }

    pub fn connect(&self, ip: &str, port: &str) -> io::Result<()> {
        let addr = format!("{}:{}", ip, port);

        // Проверяем, есть ли уже соединение с этим адресом
        if self.connection_pool.lock().unwrap().connection_exist(ip) {
            info!("Соединение с {}:{} уже существует", ip, port);
            return Ok(());
        }

        info!("Подключение к {}:{}", ip, port);
        let mut stream = TcpStream::connect_timeout(
            &addr.parse().map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?,
            Duration::from_secs(CONNECTION_TIMEOUT)
        )?;

        // Устанавливаем таймауты
        stream.set_read_timeout(Some(Duration::from_secs(CONNECTION_TIMEOUT)))?;
        stream.set_write_timeout(Some(Duration::from_secs(CONNECTION_TIMEOUT)))?;

        if stream.local_addr()? == stream.peer_addr()? {
            let _ = stream.shutdown(Shutdown::Both);
            warn!("Локальный и удаленный адреса совпадают, соединение закрыто");
            return Ok(());
        }

        info!("Успешно подключились к {}:{}", ip, port);

        let connection_pool = self.connection_pool.clone();
        let p2p_protocol = self.p2p_protocol.clone();
        let peer_address = stream.peer_addr()?.ip().to_string();

        thread::spawn(move || {
            if let Err(e) = handle_connection(&peer_address, &mut stream, &connection_pool, &p2p_protocol, true) {
                warn!("Ошибка обработки соединения: {:?}", e);
                let _ = stream.shutdown(Shutdown::Both);
                connection_pool.lock().unwrap().remove_peer(&peer_address);
            }
        });

        Ok(())
    }

    pub fn get_peer_addresses(&self) -> Vec<String> {
        self.connection_pool.lock().unwrap().get_peer_addresses()
    }

    pub fn get_peer_protocol(&self) -> Arc<Mutex<P2PProtocol>> {
        self.p2p_protocol.clone()
    }
}

fn handle_connection(
    peer_address: &str,
    stream: &mut TcpStream,
    connection_pool: &Arc<Mutex<ConnectionPool>>,
    p2p_protocol: &Arc<Mutex<P2PProtocol>>,
    is_connect: bool,
) -> Result<(), ServerError> {
    let mut last_message_time = Instant::now();

    // Если это исходящее соединение, сначала отправляем рукопожатие
    if is_connect {
        send_handshake(stream)?;
    }

    // Читаем рукопожатие (для входящих) или ответное рукопожатие (для исходящих)
    read_handshake(stream, peer_address, connection_pool, &mut last_message_time)?;

    // Для входящих соединений отправляем ответное рукопожатие после получения
    if !is_connect {
        send_handshake(stream)?;
    }

    info!("Авторизованный клиент подключен с адреса {}", peer_address);

    // Добавляем пир в пул соединений, если его еще нет
    if !connection_pool.lock().unwrap().connection_exist(peer_address) {
        // Устанавливаем постоянные таймауты для долгосрочного соединения
        stream.set_read_timeout(Some(Duration::from_secs(TIMEOUT_SECONDS / 10)))?;
        stream.set_write_timeout(Some(Duration::from_secs(30)))?;

        connection_pool.lock().unwrap().add_peer(peer_address.to_string(), stream.try_clone()?);
    }

    // Отправляем список пиров
    p2p_protocol.lock().unwrap().response_peers();

    // Если это исходящее соединение, запрашиваем первое сообщение
    if is_connect {
        p2p_protocol.lock().unwrap().request_first_message();
    }

    // Основной цикл обработки сообщений
    monitor_connection(peer_address, stream, connection_pool, p2p_protocol, &mut last_message_time)
}

fn send_handshake(stream: &mut TcpStream) -> io::Result<()> {
    stream.write_all(HANDSHAKE_MESSAGE.as_bytes())?;
    stream.flush()?;
    debug!("Отправлено рукопожатие: {}", HANDSHAKE_MESSAGE.trim());
    Ok(())
}

fn read_handshake(
    stream: &mut TcpStream,
    peer_address: &str,
    connection_pool: &Arc<Mutex<ConnectionPool>>,
    last_message_time: &mut Instant,
) -> Result<(), ServerError> {
    debug!("Ожидание рукопожатия от {}", peer_address);

    let mut buffer = vec![0; BUFFER_SIZE];
    let mut handshake_data = String::new();

    // Устанавливаем короткий таймаут для рукопожатия
    stream.set_read_timeout(Some(Duration::from_secs(CONNECTION_TIMEOUT)))?;

    let start_time = Instant::now();

    while handshake_data.trim() != HANDSHAKE_MESSAGE.trim() {
        // Проверяем таймаут рукопожатия
        if start_time.elapsed() >= Duration::from_secs(CONNECTION_TIMEOUT) {
            warn!("Таймаут рукопожатия для {}", peer_address);
            connection_pool.lock().unwrap().remove_peer(peer_address);
            return Err(ServerError::Timeout(peer_address.to_string()));
        }

        let n = match stream.read(&mut buffer) {
            Ok(0) => {
                warn!("Соединение закрыто пиром {} во время рукопожатия", peer_address);
                connection_pool.lock().unwrap().remove_peer(peer_address);
                return Err(ServerError::ConnectionClosed(peer_address.to_string()));
            }
            Ok(n) => n,
            Err(e) if e.kind() == io::ErrorKind::WouldBlock || e.kind() == io::ErrorKind::TimedOut => {
                // Таймаут чтения, проверяем снова
                continue;
            }
            Err(e) => {
                warn!("Ошибка чтения данных от {}: {:?}", peer_address, e);
                connection_pool.lock().unwrap().remove_peer(peer_address);
                return Err(ServerError::IoError(e));
            }
        };

        *last_message_time = Instant::now();
        let chunk = String::from_utf8_lossy(&buffer[..n]);
        handshake_data.push_str(&chunk);

        // Проверяем, содержит ли полученные данные рукопожатие
        if handshake_data.trim() == HANDSHAKE_MESSAGE.trim() {
            debug!("Рукопожатие успешно с {}", peer_address);
            return Ok(());
        }

        // Если получили слишком много данных и все еще нет рукопожатия, это проблема
        if handshake_data.len() > BUFFER_SIZE * 2 {
            warn!("Получено слишком много данных без корректного рукопожатия от {}", peer_address);
            connection_pool.lock().unwrap().remove_peer(peer_address);
            return Err(ServerError::InvalidHandshake(peer_address.to_string()));
        }
    }

    Ok(())
}

fn monitor_connection(
    peer_address: &str,
    stream: &mut TcpStream,
    connection_pool: &Arc<Mutex<ConnectionPool>>,
    p2p_protocol: &Arc<Mutex<P2PProtocol>>,
    last_message_time: &mut Instant,
) -> Result<(), ServerError> {
    let mut buffer = vec![0; BUFFER_SIZE];
    let mut accumulated_data = String::new();

    loop {
        // Проверяем таймаут активности
        if last_message_time.elapsed() >= Duration::from_secs(TIMEOUT_SECONDS) {
            info!("Клиент {} неактивен в течение {} секунд, отключение", peer_address, TIMEOUT_SECONDS);
            connection_pool.lock().unwrap().remove_peer(peer_address);
            return Err(ServerError::Timeout(peer_address.to_string()));
        }

        // Пинг каждые 5 минут, чтобы поддерживать соединение активным
        if last_message_time.elapsed() >= Duration::from_secs(300) && !accumulated_data.is_empty() {
            debug!("Отправка пинг-сообщения для {}", peer_address);
            if let Err(e) = stream.write_all(b"PING\n") {
                warn!("Ошибка отправки пинга для {}: {:?}", peer_address, e);
                connection_pool.lock().unwrap().remove_peer(peer_address);
                return Err(ServerError::IoError(e));
            }
            *last_message_time = Instant::now();
        }

        let n = match stream.read(&mut buffer) {
            Ok(0) => {
                info!("Соединение закрыто пиром: {}", peer_address);
                connection_pool.lock().unwrap().remove_peer(peer_address);
                return Err(ServerError::ConnectionClosed(peer_address.to_string()));
            }
            Ok(n) => n,
            Err(e) if e.kind() == io::ErrorKind::WouldBlock || e.kind() == io::ErrorKind::TimedOut => {
                // Таймаут чтения - это нормально для неблокирующего чтения
                thread::sleep(Duration::from_millis(100));
                continue;
            }
            Err(e) => {
                warn!("Ошибка чтения данных от {}: {:?}", peer_address, e);
                connection_pool.lock().unwrap().remove_peer(peer_address);
                return Err(ServerError::IoError(e));
            }
        };

        *last_message_time = Instant::now();
        accumulated_data.push_str(&String::from_utf8_lossy(&buffer[..n]));

        // Обрабатываем полные сообщения
        while let Some((message, remaining)) = extract_message(&accumulated_data) {
            debug!("Получено новое сообщение от {}: {}", peer_address, message);

            // Обновляем время последнего сообщения у пира
            // connection_pool.lock().unwrap().update_peer_timestamp(peer_address);

            // Обрабатываем сообщение в P2P протоколе
            p2p_protocol.lock().unwrap().handle_message(&message);

            accumulated_data = remaining;
        }

        // Если буфер накопленных данных слишком большой, но нет полных сообщений,
        // возможно клиент отправляет мусор - обрезаем буфер
        if accumulated_data.len() > BUFFER_SIZE * 10 {
            warn!("Слишком много данных без полных сообщений от {}, обрезаем буфер", peer_address);
            accumulated_data.clear();
        }
    }
}

/// Извлекает одно сообщение из буфера, разделенное символом `\n`.
fn extract_message(data: &str) -> Option<(String, String)> {
    data.find('\n').map(|index| {
        let message = data[..index].trim().to_string();
        let remaining = data[(index + 1)..].to_string();
        (message, remaining)
    })
}