use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

// Структура для хранения информации о сервере
pub struct Server {
    ip: String, // IP-адрес
    port: u16,  // Порт
}

impl Server {
    // Создает новый сервер с заданным IP и портом
    pub fn new(ip: &str, port: u16) -> Self {
        Server {
            ip: ip.to_string(),
            port,
        }
    }

    // Запускает сервер
    pub async fn run(&self) {
        let (tx, _) = broadcast::channel(100); // Создание канала вещания с буфером на 100 сообщений
        let addr = format!("{}:{}", self.ip, self.port); // Форматирование адреса
        let listener = TcpListener::bind(&addr).await.expect("Failed to bind address"); // Привязка сервера к порту

        println!("Server running on {}", addr);

        loop {
            let (mut stream, _) = listener.accept().await.expect("Failed to accept connection"); // Принятие входящего подключения
            let tx = tx.clone(); // Клонирование отправителя сообщений
            let (reader, mut writer) = stream.split(); // Разделение потока на чтение и запись
            let mut reader = tokio::io::BufReader::new(reader).lines(); // Буферизированное чтение строк

            // Запрос псевдонима у нового клиента
            writer.write_all(b"> Choose your nickname: ").await.expect("Failed to write to stream"); // Отправка запроса на выбор псевдонима
            writer.flush().await.expect("Failed to flush stream"); // Обеспечивает, что все данные отправлены

            // Чтение псевдонима от клиента
            let nickname = if let Some(nick) = reader.next_line().await.ok().flatten() {
                nick
            } else {
                continue; // Если не удалось получить псевдоним, продолжить ожидание нового подключения
            };

            // Обработка подключения клиента
            let tx = tx.clone(); // Клонирование отправителя сообщений
            let rx = tx.subscribe(); // Подписка на канал вещания
            tokio::spawn(handle_connection(stream, nickname, tx, rx)); // Создание асинхронной задачи для обработки подключения
        }
    }
}

// Обрабатывает подключение клиента
async fn handle_connection(
    mut stream: TcpStream, // Поток для чтения и записи данных
    nickname: String, // Псевдоним пользователя
    tx: broadcast::Sender<String>, // Отправитель сообщений для вещания
    mut rx: broadcast::Receiver<String>, // Получатель сообщений для вещания
) {
    let (reader, mut writer) = stream.split(); // Разделяет поток на чтение и запись
    let mut reader = tokio::io::BufReader::new(reader).lines(); // Буферизированное чтение строк

    // Уведомление о новом подключении
    let join_message = format!("{} just joined", nickname);
    tx.send(join_message.clone()).unwrap();
    display_message(&join_message); // Отображение сообщения на сервере

    // Отправка приветственного сообщения новому пользователю
    let welcome_message = format!(
        "===\n✨ Welcome {}!\n\nThere are {} user(s) here beside you\n\nHelp:\n - Type anything to chat\n - /list will list all the connected users\n - /quit will disconnect you\n===",
        nickname, 0 // Место для количества пользователей (временно 0)
    );
    writer.write_all(welcome_message.as_bytes()).await.unwrap(); // Отправка сообщения
    writer.flush().await.unwrap(); // Обеспечивает, что все данные отправлены

    // Основной цикл обработки входящих сообщений
    loop {
        tokio::select! {
            line = reader.next_line() => { // Чтение следующей строки из входящего потока
                match line {
                    Ok(Some(message)) => { // Успешное чтение строки
                        if message == "/quit" { // Если сообщение /quit
                            let quit_message = format!("{} has quit", nickname);
                            tx.send(quit_message.clone()).unwrap(); // Уведомление об отключении
                            display_message(&quit_message); // Отображение сообщения на сервере
                            break; // Выход из цикла обработки сообщений
                        } else if message == "/list" { // Если сообщение /list
                            let users = tx.receiver_count(); // Получение количества подключенных пользователей (временно)
                            let list_message = format!("===\nCurrently connected users:\n - {} (you)\n===\n", nickname);
                            writer.write_all(list_message.as_bytes()).await.unwrap(); // Отправка списка пользователей
                            writer.flush().await.unwrap(); // Обеспечивает, что все данные отправлены
                        } else { // Для остальных сообщений
                            let chat_message = format!("[{}] {}", nickname, message);
                            tx.send(chat_message.clone()).unwrap(); // Отправка сообщения в канал вещания
                            display_message(&chat_message); // Отображение сообщения на сервере
                        }
                    }
                    Ok(None) | Err(_) => break, // Прерывание цикла при ошибке или завершении потока
                }
            }
            msg = rx.recv() => { // Получение сообщения из канала вещания
                match msg {
                    Ok(msg) => {
                        writer.write_all(msg.as_bytes()).await.unwrap(); // Отправка сообщения пользователю
                        writer.flush().await.unwrap(); // Обеспечивает, что все данные отправлены
                    }
                    Err(_) => break, // Прерывание цикла при ошибке получения сообщения
                }
            }
        }
    }

    // Завершение соединения и уведомление об отключении
    let quit_message = format!("{} has quit", nickname);
    tx.send(quit_message.clone()).unwrap();
    display_message(&quit_message); // Отображение сообщения на сервере
}

// Функция для отображения сообщений на сервере
fn display_message(message: &str) {
    println!("{}", message);
}
