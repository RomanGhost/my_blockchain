use sha2::{Digest, Sha512};
use crate::block::Block;

pub struct BlockChain{
    pub(crate) chain:Vec<Block>,
}
impl BlockChain{
    pub fn new() -> BlockChain{
        BlockChain{
            chain:vec![],
        }
    }

    pub fn add_block(&mut self, block:Block){
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

    pub fn len(&self) -> usize{
        self.chain.len()
    }

    pub fn proof_of_work(&mut self){
        let last_block = self.get_last_block();
        match last_block {
            Ok(b) => self.proof_of_work_block(b),
            Err(e) => println!("error parsing header: {e:?}"),
        };
    }

    fn valid_block(&self, block:&Block) -> bool{
        let block_json = block.to_json();
        let hash_res = hash_string(block_json);
        let start_with = "00000";
        &hash_res[..start_with.len()] == start_with
    }

    fn proof_of_work_block(&mut self, last_block:Block){
        let mut i:u64 = 1;
        let last_block_hash = hash_string(last_block.to_json());

        loop{
            let block = Block::new(self.chain.len(), vec![], last_block_hash.clone(), last_block.get_nonce()+i);
            if self.valid_block(&block){
                println!("{}", i);
                self.chain.push(block);
                break;
            }
            i += 1;
        };

    }
}

pub fn hash_string(object: String) -> String {
    let mut hasher = Sha512::new();
    hasher.update(object);
    let result = hasher.finalize();
    let hex_string = format!("{:x}", result);
    hex_string
}