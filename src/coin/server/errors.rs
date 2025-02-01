use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("IO ошибка: {0}")]
    Io(#[from] std::io::Error),
    #[error("Ошибка рукопожатия: {0}")]
    Handshake(String),
    #[error("Клиент неактивен: {0}")]
    Timeout(String),
}