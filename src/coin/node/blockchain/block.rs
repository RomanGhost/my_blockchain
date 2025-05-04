use std::fmt;
use std::fmt::Formatter;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};

use crate::coin::node::blockchain::transaction::SerializedTransaction;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block{
    id:usize,
    time_create: i64,
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
        Block{ id, time_create: Utc::now().timestamp(), transactions, previous_hash, nonce}
    }

    pub fn force_new(
         id:usize,
         time_create: i64,
         transactions: Vec<SerializedTransaction>,
         previous_hash:String,
         nonce:u64
    ) -> Block{
        Block{ id, time_create, transactions, previous_hash, nonce}
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

    pub fn get_datetime(&self) -> i64 {
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use crate::coin::node::blockchain::transaction::SerializedTransaction;

    fn sample_transaction() -> SerializedTransaction {
        SerializedTransaction::new(
            "sender_base64".to_string(),
            "seller_base64".to_string(),
            "buyer_base64".to_string(),
            "Test message".to_string(),
            123.45,
        )
    }

    #[test]
    fn test_block_creation() {
        let transactions = vec![sample_transaction()];
        let block = Block::new(1, transactions.clone(), "prev_hash".to_string(), 42);

        assert_eq!(block.get_id(), 1);
        assert_eq!(block.get_nonce(), 42);
        assert_eq!(block.get_previous_hash(), "prev_hash");
        assert_eq!(block.get_transactions(), &transactions);
    }

    #[test]
    fn test_block_hash_is_deterministic() {
        let transactions = vec![sample_transaction()];
        let block = Block::new(2, transactions, "abc123".to_string(), 999);

        let hash1 = block.get_hash();
        let hash2 = block.get_hash();

        assert_eq!(hash1, hash2, "Хэш должен быть одинаков при одинаковом содержимом");
    }

    #[test]
    fn test_block_json_serialization() {
        let transactions = vec![sample_transaction()];
        let block = Block::new(3, transactions, "prev".to_string(), 777);

        let json = block.to_json();
        let deserialized: Block = serde_json::from_str(&json).expect("Ошибка десериализации");

        assert_eq!(deserialized.get_id(), 3);
        assert_eq!(deserialized.get_nonce(), 777);
        assert_eq!(deserialized.get_previous_hash(), "prev");
    }

    #[test]
    fn test_block_set_previous_hash() {
        let mut block = Block::new(4, vec![], "old_hash".to_string(), 0);
        block.set_previous_hash("new_hash".to_string());

        assert_eq!(block.get_previous_hash(), "new_hash");
    }

    #[test]
    fn test_block_display() {
        let block = Block::new(5, vec![], "hash123".to_string(), 12345);
        let output = format!("{}", block);

        assert!(output.contains("id: 5"));
        assert!(output.contains("previous_hash: hash123"));
        assert!(output.contains("nonce: 12345"));
    }
}
