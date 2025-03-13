use std::fmt;
use std::fmt::Formatter;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};

use crate::coin::node::blockchain::transaction::SerializedTransaction;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block{
    id:usize,
    time_create: DateTime<Utc>,
    transactions: Vec<SerializedTransaction>,
    previous_hash: String,
    nonce: u64
}
impl Block{
    pub fn new(
        id:usize,
        transactions: Vec<SerializedTransaction>,
        previous_hash:String,
        nonce:u64
    ) -> Block{
        Block{ id, time_create: Utc::now(), transactions, previous_hash, nonce}
    }

    pub fn clone(&self) -> Block{
        Block{
            id:self.id,
            time_create:self.time_create,
            transactions: self.transactions.clone(),
            previous_hash: self.previous_hash.clone(),
            nonce: self.nonce,
        }
    }

    pub fn get_hash(&self) ->String{
        let mut hasher = Sha512::new();
        hasher.update(format!("{}_{:?}_{}/{}", self.id, self.transactions, self.previous_hash, self.nonce ));

        let result = hasher.finalize();
        // Преобразование результата хэширования в строку
        format!("{:x}", result)
    }

    pub fn get_nonce(&self)->u64{
        self.nonce
    }

    pub fn to_json(&self) -> String{
        serde_json::to_string(&self).unwrap()
    }

    pub fn get_previous_hash(&self) -> String {
        self.previous_hash.clone()
    }

    pub fn get_id(&self) -> usize{
        self.id
    }

    pub fn get_datetime(&self) -> DateTime<Utc> {
        self.time_create
    }

    pub fn set_previous_hash(&mut self, last_hash:String){
        self.previous_hash = last_hash;
    }

    pub fn get_transactions(&self) -> &Vec<SerializedTransaction> {
        &self.transactions
    }
}

impl fmt::Display for Block{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "id: {}\ntime_create: {}\nprevious_hash: {}\nnonce: {}",
               self.id, self.time_create, self.previous_hash,
               self.nonce)
    }
}