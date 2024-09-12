use sha2::{Sha256, Digest};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
struct Block {
    index: u64,
    previous_hash: String,
    timestamp: u64,
    data: String,
    nonce: u64,
    hash: String,
}

impl Block {
    fn new(index: u64, previous_hash: String, data: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
        let mut block = Block {
            index,
            previous_hash,
            timestamp,
            data,
            nonce: 0,
            hash: String::new(),
        };
        block.hash = block.calculate_hash();
        block
    }

    fn calculate_hash(&self) -> String {
        let data = format!(
            "{}{}{}{}{}",
            self.index, self.previous_hash, self.timestamp, self.data, self.nonce
        );
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    fn mine_block(&mut self, difficulty: usize) {
        let target = "0".repeat(difficulty);
        while &self.hash[..difficulty] != target {
            self.nonce += 1;
            self.hash = self.calculate_hash();
        }
        println!("Block mined: {}", self.hash);
    }
}

struct Blockchain {
    chain: Vec<Block>,
    difficulty: usize,
}

impl Blockchain {
    fn new() -> Self {
        let mut blockchain = Blockchain {
            chain: Vec::new(),
            difficulty: 4,
        };
        blockchain.chain.push(Block::new(0, String::from("0"), String::from("Genesis Block")));
        blockchain
    }

    fn add_block(&mut self, data: String) {
        let previous_hash = self.chain.last().unwrap().hash.clone();
        let mut new_block = Block::new(self.chain.len() as u64, previous_hash, data);
        new_block.mine_block(self.difficulty);
        self.chain.push(new_block);
    }
}

fn main() {
    let mut blockchain = Blockchain::new();
    blockchain.add_block("Block 1 Data".to_string());
    blockchain.add_block("Block 2 Data".to_string());

    for block in &blockchain.chain {
        println!("{:?}", block);
    }
}
