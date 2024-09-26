use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json;
use crate::coin::blockchain::block::Block;
use crate::coin::blockchain::transaction::Transaction;

// Дополнительный перечисляемый тип для представления типов сообщений
#[derive(Debug, PartialEq, Eq)]
pub enum MessageType {
    Block,
    Transaction,
    Text,
    RequestLastNBlocks,      // Новый тип сообщения для запроса последних N блоков
    RequestBlocksBefore,      // Новый тип сообщения для запроса блоков до определенной даты
}

// Обобщённый тип сообщения, содержащий разные варианты
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "content")] // Добавляем тег для типа сообщения
pub enum Message {
    BlockMessage(BlockMessage),
    TransactionMessage(TransactionMessage),
    TextMessage(TextMessage),
    RequestLastNBlocksMessage(RequestLastNBlocksMessage), // Добавляем новый тип сообщения
    RequestBlocksBeforeMessage(RequestBlocksBeforeMessage), // Добавляем еще один новый тип сообщения
}

impl Message {
    // Унифицированный метод для сериализации
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    // Унифицированный метод для десериализации
    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }

    // Метод для получения типа сообщения в виде перечисления MessageType
    pub fn get_type(&self) -> MessageType {
        match self {
            Message::BlockMessage(_) => MessageType::Block,
            Message::TransactionMessage(_) => MessageType::Transaction,
            Message::TextMessage(_) => MessageType::Text,
            Message::RequestLastNBlocksMessage(_) => MessageType::RequestLastNBlocks,
            Message::RequestBlocksBeforeMessage(_) => MessageType::RequestBlocksBefore,
        }
    }

    // Унифицированные методы get_id и set_id для всех вариантов сообщения
    pub fn get_id(&self) -> u64 {
        match self {
            Message::BlockMessage(msg) => msg.get_id(),
            Message::TransactionMessage(msg) => msg.get_id(),
            Message::TextMessage(msg) => msg.get_id(),
            Message::RequestLastNBlocksMessage(msg) => msg.get_id(),
            Message::RequestBlocksBeforeMessage(msg) => msg.get_id(),
        }
    }

    pub fn set_id(&mut self, id: u64) {
        match self {
            Message::BlockMessage(msg) => msg.set_id(id),
            Message::TransactionMessage(msg) => msg.set_id(id),
            Message::TextMessage(msg) => msg.set_id(id),
            Message::RequestLastNBlocksMessage(msg) => msg.set_id(id),
            Message::RequestBlocksBeforeMessage(msg) => msg.set_id(id),
        }
    }
}

// Пример структуры BlockMessage с флагом force
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockMessage {
    id: u64,
    block: Block,
    time_stamp: DateTime<Utc>,
    force: bool, // Добавляем флаг для определения, является ли сообщение "force"
}

impl BlockMessage {
    pub fn new(block: Block, force: bool) -> BlockMessage {
        BlockMessage {
            id: 0,
            block,
            time_stamp: Utc::now(),
            force,
        }
    }

    pub fn get_id(&self) -> u64 {
        self.id
    }

    pub fn set_id(&mut self, id: u64) {
        self.id = id;
    }
}

// Пример структуры TransactionMessage
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransactionMessage {
    id: u64,
    transaction: Transaction,
    time_stamp: DateTime<Utc>,
}

impl TransactionMessage {
    pub fn new(transaction: Transaction) -> TransactionMessage {
        TransactionMessage {
            id: 0,
            transaction,
            time_stamp: Utc::now(),
        }
    }

    pub fn get_id(&self) -> u64 {
        self.id
    }

    pub fn set_id(&mut self, id: u64) {
        self.id = id;
    }
}

// Пример структуры TextMessage
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TextMessage {
    id: u64,
    message: String,
    time_stamp: DateTime<Utc>,
}

impl TextMessage {
    pub fn new(message: String) -> TextMessage {
        TextMessage {
            id: 0,
            message,
            time_stamp: Utc::now(),
        }
    }

    pub fn get_id(&self) -> u64 {
        self.id
    }

    pub fn set_id(&mut self, id: u64) {
        self.id = id;
    }

    pub fn get_text(&self) -> String {
        self.message.clone()
    }
}

// Новый тип сообщения для запроса последних N блоков
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RequestLastNBlocksMessage {
    id: u64,
    n: usize, // Количество блоков, которые необходимо запросить
}

impl RequestLastNBlocksMessage {
    pub fn new(n: usize) -> RequestLastNBlocksMessage {
        RequestLastNBlocksMessage { id: 0, n }
    }

    pub fn get_id(&self) -> u64 {
        self.id
    }

    pub fn set_id(&mut self, id: u64) {
        self.id = id;
    }
}

// Новый тип сообщения для запроса блоков до определенной даты
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RequestBlocksBeforeMessage {
    id: u64,
    time_stamp: DateTime<Utc>, // Запрашиваем все блоки до этого времени
}

impl RequestBlocksBeforeMessage {
    pub fn new(time_stamp: DateTime<Utc>) -> RequestBlocksBeforeMessage {
        RequestBlocksBeforeMessage { id: 0, time_stamp }
    }

    pub fn get_id(&self) -> u64 {
        self.id
    }

    pub fn set_id(&mut self, id: u64) {
        self.id = id;
    }
}
