mod transaction;
mod block;
mod blockchain;

use sha2::Digest;
use blockchain::{Blockchain, hash_string};

fn main() {
    let mut blockchain = Blockchain::new();
    blockchain.create_first_block();
    let last_block = blockchain.get_last_block();

    match last_block {
    // Ok(v) => v.get_hash(),
    Ok(v) => println!("working with version: {}", v.get_hash()),
    Err(e) => println!("error parsing header: {e:?}"),
    };
    while blockchain.len() < 5{
        blockchain.proof_of_work();
        println!("Len: {}", blockchain.chain.len());
    }
}