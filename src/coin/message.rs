use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::coin::blockchain::block::Block;
use serde_json;
use crate::coin::blockchain::transaction::Transaction;

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockMessage {
    id: u64,
    block: Block,
    time_stamp: DateTime<Utc>,
}

impl BlockMessage {
    pub fn new(id: u64, last_block: Block) -> BlockMessage {
        BlockMessage {
            id,
            block: last_block,
            time_stamp: Utc::now(),
        }
    }

    pub fn to_json(&self) -> String{
        serde_json::to_string(&self).unwrap()
    }
}


#[derive(Serialize, Deserialize, Debug)]
pub struct TransactionMessage {
    id: u64,
    transaction: Transaction,
    time_stamp: DateTime<Utc>,
}

impl TransactionMessage {
    pub fn new(id: u64, new_transaction:Transaction) -> TransactionMessage {
        TransactionMessage {
            id,
            transaction: new_transaction,
            time_stamp: Utc::now(),
        }
    }

    pub fn to_json(&self) -> String{
        serde_json::to_string(&self).unwrap()
    }
}