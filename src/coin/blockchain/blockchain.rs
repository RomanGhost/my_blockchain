use chrono::{DateTime, Utc};
use sha2::{Digest, Sha512};

use crate::coin::blockchain::block::Block;
use crate::coin::blockchain::transaction::SerializedTransaction;

pub struct Blockchain {
    pub chain:Vec<Block>,
    nonce_iteration: u64,
}
impl Blockchain {
    pub fn new() -> Blockchain {
        Blockchain {
            chain:vec![],
            nonce_iteration: 0
        }
    }

    pub fn add_block(&mut self, block:Block){
        let mut block = block;
        match self.get_last_block(){
            Ok(last_block) =>{
                if block.get_previous_hash() == last_block.get_hash() {
                    self.chain.push(block);
                }
            }
            Err(e)=>{
                eprintln!("Error adding block: {e}");
            }
        };
        // println!("Количество блоков в блокчейн: {}", self.chain.len());
    }

    pub fn add_force_block(&mut self, block:Block){
        // let mut block = block;
        // match self.get_last_block(){
        //     Ok(last_block) =>{
        //         if last_block.get_id() >= block.get_id() {
        //             println!("Не можем добавить новый блок т.к. id последнего блока больше");
        //             return;
        //         }
        //     }
        //     Err(e)=>{
        //         eprintln!("Error adding block: {e}");
        //     }
        // };
        self.chain.push(block);
    }

    pub fn get_last_block(&self) -> Result<Block, &'static str>{
        let chain_len = self.chain.len();
        if chain_len <= 0 {
            Err("chain is Empty")
        } else{
            Ok(self.chain[chain_len -1].clone())
        }
    }

    pub fn create_first_block(&mut self) {
        let word = "First block"; //Сделать это переменной окружения
        let mut hasher = Sha512::new();
        hasher.update(word);
        let result = hasher.finalize();
        let hex_string = format!("{:x}", result);

        let block = Block::new(1, vec![], hex_string, 0);
        self.add_force_block(block);
    }

    pub fn len(&self) -> usize{
        self.chain.len()
    }

    fn valid_block(&self, block:&Block) -> bool{
        let block_hash = block.get_hash();
        let start_with = "000";

        &block_hash[..start_with.len()] == start_with
    }

    pub fn proof_of_work(&mut self, transactions: Vec<SerializedTransaction>) -> bool {
        let last_block = self.get_last_block();
        let mut result = false;
        match last_block {
            Ok(b) => result = self._proof_of_work(b, transactions),
            Err(_) => {
                eprintln!("Блокчейн пуст");
                self.create_first_block();
                self.proof_of_work(transactions);
            },
        };
        result
    }

    fn _proof_of_work(&mut self, last_block: Block, transactions: Vec<SerializedTransaction>) -> bool {
        let last_block_hash = last_block.get_hash();
        let block = Block::new(
            last_block.get_id() + 1,
            transactions,
            last_block_hash.clone(),
            self.nonce_iteration,
        );

        let validation = self.valid_block(&block);
        if validation {
            println!("Create new block with id: {}", block.get_id());
            self.chain.push(block);
            self.nonce_iteration = 0;
        }
        self.nonce_iteration += 1;

        validation

    }

    pub fn get_blocks_after(&self, datetime: DateTime<Utc>) -> Vec<Block> {
        let result: Vec<Block> = self.chain
            .iter()
            .filter(|&block| datetime < block.get_datetime()) // Фильтруем блоки по дате
            .cloned() // Преобразуем `&Block` в `Block` с помощью `cloned`
            .collect(); // Собираем результат в `Vec<Block>`
        result
    }

    pub fn get_last_n_blocks(&self, n:usize) -> Vec<Block> {
        let last_n = &self.chain[self.chain.len().saturating_sub(n)..];
        last_n.iter().cloned().collect()
    }

    pub fn clear_nonce(&mut self) {
        self.nonce_iteration = 0;
    }
}
