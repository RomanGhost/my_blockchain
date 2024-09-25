use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::coin::blockchain::block::Block;
use crate::coin::blockchain::transaction::Transaction;
use serde_json;

// Обобщённый тип сообщения, содержащий разные варианты
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "content")]  // Добавляем тег для типа сообщения
pub enum Message {
    BlockMessage(BlockMessage),
    TransactionMessage(TransactionMessage),
    TextMessage(TextMessage),
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

    // Унифицированные методы get_id и set_id для всех вариантов сообщения
    pub fn get_id(&self) -> u64 {
        match self {
            Message::BlockMessage(msg) => msg.get_id(),
            Message::TransactionMessage(msg) => msg.get_id(),
            Message::TextMessage(msg) => msg.get_id(),
        }
    }

    pub fn set_id(&mut self, id: u64) {
        match self {
            Message::BlockMessage(msg) => msg.set_id(id),
            Message::TransactionMessage(msg) => msg.set_id(id),
            Message::TextMessage(msg) => msg.set_id(id),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockMessage {
    id: u64,
    block: Block,
    time_stamp: DateTime<Utc>,
}

impl BlockMessage {
    pub fn new(block: Block) -> BlockMessage {
        BlockMessage {
            id:0,
            block,
            time_stamp: Utc::now(),
        }
    }

    // Методы get_id и set_id
    pub fn get_id(&self) -> u64 {
        self.id
    }

    pub fn set_id(&mut self, id: u64) {
        self.id = id;
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TransactionMessage {
    id: u64,
    transaction: Transaction,
    time_stamp: DateTime<Utc>,
}

impl TransactionMessage {
    pub fn new(transaction: Transaction) -> TransactionMessage {
        TransactionMessage {
            id:0,
            transaction,
            time_stamp: Utc::now(),
        }
    }

    // Методы get_id и set_id
    pub fn get_id(&self) -> u64 {
        self.id
    }

    pub fn set_id(&mut self, id: u64) {
        self.id = id;
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TextMessage {
    id: u64,
    message: String,
    time_stamp: DateTime<Utc>,
}

impl TextMessage {
    pub fn new(message: String) -> TextMessage {
        TextMessage {
            id:0,
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

    pub fn get_text(&self) -> String{
        self.message.clone()
    }
}
