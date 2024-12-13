use std::sync::{Arc, Mutex};
use std::collections::BinaryHeap;
use crate::coin::blockchain::blockchain::Blockchain;
use crate::coin::blockchain::transaction::SerializedTransaction;

pub fn initialize_blockchain() -> (Arc<Mutex<Blockchain>>, Arc<Mutex<BinaryHeap<SerializedTransaction>>>) {
    let blockchain = Arc::new(Mutex::new(Blockchain::new()));
    let queue = Arc::new(Mutex::new(BinaryHeap::new()));
    (blockchain, queue)
}

pub fn count_wallet_amount(my_public_key: String, blockchain: &Blockchain) -> f64 {
    let chain = &blockchain.chain;
    let mut amount = 0.0;
    for block in chain {
        let transactions = block.get_transactions();
        for transaction in transactions {
            if transaction.get_sender() == my_public_key {
                amount -= transaction.transfer;
            }
        }
    }
    amount
}
