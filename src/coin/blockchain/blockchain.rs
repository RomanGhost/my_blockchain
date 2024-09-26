use chrono::{DateTime, Utc};
use sha2::{Digest, Sha512};

use crate::coin::blockchain::block::Block;

pub struct Blockchain {
    pub chain:Vec<Block>,
}
impl Blockchain {
    pub fn new() -> Blockchain {
        Blockchain {
            chain:vec![],
        }
    }

    pub fn add_block(&mut self, block:Block){
        if block.get_previous_hash() == self.get_last_block().unwrap().get_hash() {
            self.chain.push(block);
        }
    }

    pub fn force_add_block(&mut self, block:Block){
        let mut block = block;
        match self.get_last_block(){
            Ok(last_block) =>{
                block.set_previous_hash(last_block.get_hash());
            }
            Err(e)=>{
                eprintln!("Error adding block: {e}");
            }
        };

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
        self.add_block(block);
    }

    pub fn len(&self) -> usize{
        self.chain.len()
    }

    fn valid_block(&self, block:&Block) -> bool{
        let block_hash = block.get_hash();
        let start_with = "000";
        
        &block_hash[..start_with.len()] == start_with
    }

    pub fn proof_of_work(&mut self){
        let last_block = self.get_last_block();
        match last_block {
            Ok(b) => self._proof_of_work(b),
            Err(e) => println!("error parsing header: {e:?}"),
        };
    }

    fn _proof_of_work(&mut self, last_block:Block){
        let mut i:u64 = 0;
        let last_block_hash = last_block.get_hash();

        loop{
            let block = Block::new(last_block.get_id(), vec![], last_block_hash.clone(), last_block.get_nonce()+i);
            if self.valid_block(&block){
                println!("{}", i);
                self.chain.push(block);
                break;
            }
            i += 1;
        };
    }

    pub fn get_blocks_after(&self, datetime: DateTime<Utc>) -> Vec<Block> {
        let result: Vec<Block> = self.chain
            .iter()
            .filter(|&block| datetime < block.get_datetime()) // Фильтруем блоки по дате
            .cloned() // Преобразуем `&Block` в `Block` с помощью `cloned`
            .collect(); // Собираем результат в `Vec<Block>`
        result
    }
}
