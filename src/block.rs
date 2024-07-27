use chrono::{DateTime, Utc};
use sha2::{Sha512, Digest};

use crate::transaction::Transaction;


pub struct Block{
    id:usize,
    time_create: DateTime<Utc>,
    transactions: Vec<Transaction>,
    previous_hash: String,
    nonce: u64
}
impl Block{
    pub fn new(
        id:usize,
        transactions:Vec<Transaction>,
        previous_hash:String,
        nonce:u64
    ) -> Block{
        Block{ id, time_create: Utc::now(), transactions, previous_hash, nonce}
    }

    pub fn clone(&self) -> Block{
        Block{
            id:self.id,
            time_create:self.time_create,
            transactions: self.transactions.iter().map(|t| Transaction{id:t.id}).collect::<Vec<Transaction>>(),
            previous_hash: self.previous_hash.clone(),
            nonce: self.nonce,
        }
    }

    pub fn to_string(&self) ->String{
        format!("id: {}\ntime_create: {}\nprevious_hash: {}\nnonce: {}",
                self.id, self.time_create, self.previous_hash,
                self.nonce)
    }

    pub fn to_json(&self) ->String{
        format!("
{{
\"id\":{},
\"time_create\":\"{}\",
\"transactions\":{:?},
\"previous_hash\":\"{}\",
\"nonce\":{:X}
}}
",
                self.id,
                self.time_create,
                self.transactions.iter().map(|t| t.id).collect::<Vec<u64>>(),
                self.previous_hash,
                self.nonce,
        )
    }

    pub fn get_hash(&self) ->String{
        let mut hasher = Sha512::new();
        hasher.update(format!("{}", self.to_json()));

        let result = hasher.finalize();
        // Преобразование результата хэширования в строку
        format!("{:x}", result)
    }

    pub fn get_nonce(&self)->u64{
        self.nonce
    }
}