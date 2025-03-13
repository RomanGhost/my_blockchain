use chrono::{DateTime, Utc};
use sha2::{Digest, Sha512};

use crate::coin::node::blockchain::block::Block;

pub struct Blockchain {
    pub chain: Vec<Block>,
    nonce_iteration: u64,
}
impl Blockchain {
    pub fn new() -> Blockchain {
        Blockchain {
            chain: Vec::new(),
            nonce_iteration: 0,
        }
    }

    pub fn add_block(&mut self, block: Block) -> Result<Block, String> {
        let mut block = block;
        if !Blockchain::is_valid_block(&block) {
            return Err("Hash didn't valid".to_string())
        }
        if let Ok(last_block) = self.get_last_block() {
            if block.get_previous_hash() == last_block.get_hash() {
                self.chain.push(block.clone());
                Ok(block)
            } else {
                Err("Хеши не совпадают".to_string())
            }
        } else {
            Err("chain is empty".to_string())
        }
    }

    pub fn add_force_block(&mut self, block: Block) {
        self.chain.push(block);
    }

    pub fn get_last_block(&self) -> Result<Block, &'static str> {
        if let Some(block) = self.chain.last() {
            Ok(block.clone())
        } else {
            Err("chain is empty")
        }
    }

    pub fn create_first_block(&mut self) {
        let word = "First block";
        let mut hasher = Sha512::new();
        hasher.update(word);
        let result = hasher.finalize();
        let hex_string = format!("{:x}", result);

        let block = Block::new(1, Vec::new(), hex_string, 0);
        self.add_force_block(block);
    }

    pub fn len(&self) -> usize {
        self.chain.len()
    }

    pub fn is_valid_block(block: &Block) -> bool {
        block.get_hash().starts_with("000")
    }

    pub fn get_blocks_after(&self, datetime: DateTime<Utc>) -> Vec<Block> {
        self.chain
            .iter()
            .filter(|block| datetime < block.get_datetime())
            .cloned()
            .collect()
    }

    pub fn get_blocks_before(&self, datetime: DateTime<Utc>) -> Vec<Block> {
        self.chain
            .iter()
            .filter(|block| datetime > block.get_datetime())
            .cloned()
            .collect()
    }

    pub fn get_last_n_blocks(&self, n: usize) -> Vec<Block> {
        self.chain
            .iter()
            .take(n)
            .cloned()
            .collect()
    }

    pub fn clear_nonce(&mut self) {
        self.nonce_iteration = 0;
    }
}

pub fn validate_chain(blockchain: &Vec<Block>, new_chain: &Vec<Block>) -> bool {
    for i in 1..new_chain.len() {
        let current_block = &new_chain[i];
        let previous_block = &new_chain[i - 1];

        // Проверка корректности ссылок на предыдущие блоки
        if current_block.get_previous_hash() != previous_block.get_hash() {
            return false;
        }

        // Дополнительная проверка хешей и PoW
        if !Blockchain::is_valid_block(current_block) {
            return false;
        }
    }
    true
}