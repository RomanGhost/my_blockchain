use chrono::{DateTime, Utc};
use sha2::{Digest, Sha512};

use crate::coin::blockchain::block::Block;
use crate::coin::blockchain::transaction::SerializedTransaction;

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

    pub fn add_block(&mut self, block: Block) {
        let mut block = block;
        if let Ok(last_block) = self.get_last_block() {
            if block.get_previous_hash() == last_block.get_hash() {
                self.chain.push(block);
            } else {
                println!("Хеши не совпадают");
            }
        } else {
            eprintln!("Error adding block: chain is empty");
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

    fn valid_block(&self, block: &Block) -> bool {
        block.get_hash().starts_with("000")
    }

    pub fn proof_of_work(&mut self, transactions: Vec<SerializedTransaction>) -> bool {
        if let Ok(last_block) = self.get_last_block() {
            self._proof_of_work(last_block, transactions)
        } else {
            eprintln!("Blockchain is empty, creating the first block.");
            self.create_first_block();
            self.proof_of_work(transactions)
        }
    }

    fn _proof_of_work(&mut self, last_block: Block, transactions: Vec<SerializedTransaction>) -> bool {
        let last_block_hash = last_block.get_hash();
        let block = Block::new(
            last_block.get_id() + 1,
            transactions,
            last_block_hash.clone(),
            self.nonce_iteration,
        );

        if self.valid_block(&block) {
            println!("Create new block with id: {}", block.get_id());
            self.chain.push(block);
            self.nonce_iteration = 0;
            return true;
        }

        self.nonce_iteration += 1;
        false
    }

    pub fn get_blocks_after(&self, datetime: DateTime<Utc>) -> Vec<Block> {
        self.chain
            .iter()
            .filter(|block| datetime < block.get_datetime())
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