use serde::{Deserialize, Serialize};
use serde_json;
use crate::coin::server::protocol::message::{request, response};

// Обобщённый тип сообщения, содержащий разные варианты
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "content")] // Добавляем тег для типа сообщения
pub enum Message {
    RawMessage(String),

    ResponseBlockMessage(response::BlockMessage),
    ResponseTransactionMessage(response::TransactionMessage),
    ResponseTextMessage(response::TextMessage),
    ResponseMessageInfo(response::MessageAnswerFirstInfo),
    ResponseChainMessage(response::ChainMessage),
    ResponsePeerMessage(response::PeerMessage),

    RequestLastNBlocksMessage(request::LastNBlocksMessage),
    RequestBlocksBeforeMessage(request::BlocksBeforeMessage),
    RequestMessageInfo(request::MessageFirstInfo),
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
            Message::RawMessage(msg) => 0,

            Message::ResponseBlockMessage(msg) => msg.get_id(),
            Message::ResponseTransactionMessage(msg) => msg.get_id(),
            Message::ResponseTextMessage(msg) => msg.get_id(),
            Message::ResponseMessageInfo(msg) => msg.get_id(),
            Message::ResponseChainMessage(msg) => msg.get_id(),
            Message::ResponsePeerMessage(msg) => msg.get_id(),

            Message::RequestLastNBlocksMessage(msg) => msg.get_id(),
            Message::RequestBlocksBeforeMessage(msg) => msg.get_id(),
            Message::RequestMessageInfo(msg) => msg.get_id(),
        }
    }

    pub fn set_id(&mut self, id: u64) {
        match self {
            Message::RawMessage(msg) => (),

            Message::ResponseBlockMessage(msg) => msg.set_id(id),
            Message::ResponseTransactionMessage(msg) => msg.set_id(id),
            Message::ResponseTextMessage(msg) => msg.set_id(id),
            Message::ResponseMessageInfo(msg) => msg.set_id(id),
            Message::ResponseChainMessage(msg) => msg.set_id(id),
            Message::ResponsePeerMessage(msg) => msg.set_id(id),

            Message::RequestLastNBlocksMessage(msg) => msg.set_id(id),
            Message::RequestBlocksBeforeMessage(msg) => msg.set_id(id),
            Message::RequestMessageInfo(msg) => msg.set_id(id),
        }
    }
}