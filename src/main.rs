mod transaction;
mod block;
mod block_chain;

use sha2::{Sha512, Digest};
use block::Block;
use crate::block_chain::{BlockChain, hash_string};

fn main() {
    let hex_string = hash_string("Hello world".to_string());
    let block = Block::new(1, vec![], hex_string, 29);

    let mut block_chain = BlockChain::new();
    block_chain.add_block(block);
    let last_block = block_chain.get_last_block();

    match last_block {
    // Ok(v) => v.get_hash(),
    Ok(v) => println!("working with version: {}", v.get_hash()),
    Err(e) => println!("error parsing header: {e:?}"),
    };
    while block_chain.len() < 10{
        block_chain.proof_of_work();
        println!("Len: {}", block_chain.chain.len());
    }
}