use std::io::Error;
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

impl ServerError {
    pub(crate) fn InvalidHandshake(p0: String) -> ServerError {
        todo!()
    }
}

impl ServerError {
    pub(crate) fn ConnectionClosed(p0: String) -> ServerError {
        todo!()
    }
}

impl ServerError {
    pub(crate) fn IoError(p0: Error) -> ServerError {
        todo!()
    }
}