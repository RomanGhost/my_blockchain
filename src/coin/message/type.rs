use serde::{Deserialize, Serialize};
use serde_json;
use crate::coin::message::request::{BlocksBeforeMessage, LastNBlocksMessage, MessageFirstInfo};
use crate::coin::message::response::{BlockMessage, MessageAnswerFirstInfo, TextMessage, TransactionMessage};

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
    ResponseBlockMessage(BlockMessage),
    ResponseTransactionMessage(TransactionMessage),
    ResponseTextMessage(TextMessage),
    ResponseMessageInfo(MessageAnswerFirstInfo),

    RequestLastNBlocksMessage(LastNBlocksMessage),
    RequestBlocksBeforeMessage(BlocksBeforeMessage),
    RequestMessageInfo(MessageFirstInfo),

}

impl Message {
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }

    // Унифицированные методы get_id и set_id для всех вариантов сообщения
    pub fn get_id(&self) -> u64 {
        match self {
            Message::ResponseBlockMessage(msg) => msg.get_id(),
            Message::ResponseTransactionMessage(msg) => msg.get_id(),
            Message::ResponseTextMessage(msg) => msg.get_id(),
            Message::RequestLastNBlocksMessage(msg) => msg.get_id(),
            Message::RequestBlocksBeforeMessage(msg) => msg.get_id(),
            Message::RequestMessageInfo(msg) => msg.get_id(),
            Message::ResponseMessageInfo(msg) => msg.get_id(),
        }
    }

    pub fn set_id(&mut self, id: u64) {
        match self {
            Message::ResponseBlockMessage(msg) => msg.set_id(id),
            Message::ResponseTransactionMessage(msg) => msg.set_id(id),
            Message::ResponseTextMessage(msg) => msg.set_id(id),
            Message::RequestLastNBlocksMessage(msg) => msg.set_id(id),
            Message::RequestBlocksBeforeMessage(msg) => msg.set_id(id),
            Message::RequestMessageInfo(msg) => msg.set_id(id),
            Message::ResponseMessageInfo(msg) => msg.set_id(id),
        }
    }
}
