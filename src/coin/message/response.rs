use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::coin::blockchain::block::Block;
use crate::coin::blockchain::transaction::{SerializedTransaction, Transaction};

// Пример структуры BlockMessage с флагом force
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockMessage {
    id: u64,
    block: Block,
    time_stamp: DateTime<Utc>,
    force: bool,
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

    pub fn is_force(&self) -> bool {
        self.force
    }

    pub fn get_block(&self) -> Block {
        self.block.clone()
    }
}

// Пример структуры TransactionMessage
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransactionMessage {
    id: u64,
    transaction: SerializedTransaction,
    time_stamp: DateTime<Utc>,
}

impl TransactionMessage {
    pub fn new(transaction: SerializedTransaction) -> TransactionMessage {
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

    pub fn get_transaction(&self) -> SerializedTransaction {
        self.transaction.clone()
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MessageAnswerFirstInfo {
    id: u64
}

impl MessageAnswerFirstInfo {
    pub fn new() -> MessageAnswerFirstInfo {
        MessageAnswerFirstInfo{ id: 0, }
    }

    pub fn get_id(&self) -> u64 {
        self.id
    }

    pub fn set_id(&mut self, id: u64) {
        self.id = id;
    }
}