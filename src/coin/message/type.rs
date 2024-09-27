use serde::{Deserialize, Serialize};
use serde_json;
use crate::coin::message::request::{BlocksBeforeMessage, LastNBlocksMessage};
use crate::coin::message::response::{BlockMessage, TextMessage, TransactionMessage};

// Дополнительный перечисляемый тип для представления типов сообщений
// #[derive(Debug, PartialEq, Eq)]
// pub enum MessageType {
//     ResponseBlock,
//     ResponseTransaction,
//     ResponseText,
//     RequestLastNBlocks,      // Новый тип сообщения для запроса последних N блоков
//     RequestBlocksBefore,      // Новый тип сообщения для запроса блоков до определенной даты
// }

// Обобщённый тип сообщения, содержащий разные варианты
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "content")] // Добавляем тег для типа сообщения
pub enum Message {
    BlockMessage(BlockMessage),
    TransactionMessage(TransactionMessage),
    TextMessage(TextMessage),
    LastNBlocksMessage(LastNBlocksMessage), // Добавляем новый тип сообщения
    BlocksBeforeMessage(BlocksBeforeMessage), // Добавляем еще один новый тип сообщения
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
    // pub fn get_type(&self) -> MessageType {
    //     match self {
    //         Message::BlockMessage(_) => MessageType::ResponseBlock,
    //         Message::TransactionMessage(_) => MessageType::ResponseTransaction,
    //         Message::TextMessage(_) => MessageType::ResponseText,
    //         Message::LastNBlocksMessage(_) => MessageType::RequestLastNBlocks,
    //         Message::BlocksBeforeMessage(_) => MessageType::RequestBlocksBefore,
    //     }
    // }

    // Унифицированные методы get_id и set_id для всех вариантов сообщения
    pub fn get_id(&self) -> u64 {
        match self {
            Message::BlockMessage(msg) => msg.get_id(),
            Message::TransactionMessage(msg) => msg.get_id(),
            Message::TextMessage(msg) => msg.get_id(),
            Message::LastNBlocksMessage(msg) => msg.get_id(),
            Message::BlocksBeforeMessage(msg) => msg.get_id(),
        }
    }

    pub fn set_id(&mut self, id: u64) {
        match self {
            Message::BlockMessage(msg) => msg.set_id(id),
            Message::TransactionMessage(msg) => msg.set_id(id),
            Message::TextMessage(msg) => msg.set_id(id),
            Message::LastNBlocksMessage(msg) => msg.set_id(id),
            Message::BlocksBeforeMessage(msg) => msg.set_id(id),
        }
    }
}
