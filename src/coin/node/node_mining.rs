use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, Sender};

use log::warn;

use crate::coin::node::blockchain::block::Block;
use crate::coin::node::blockchain::blockchain::Blockchain;
use crate::coin::node::blockchain::transaction::{SerializedTransaction, Transaction};
use crate::coin::node::node_message::{BlockchainMessage, TransactionMessage};
use crate::coin::node::node_message::BlockchainMessage::BlockAdd;
use crate::coin::node::node_message::TransactionMessage::{AddTransaction, GetTransaction};

pub struct NodeMining {
    tx_blockchain: Sender<BlockchainMessage>,
    tx_transactions:Sender<TransactionMessage>,
    rx_transactions: Receiver<TransactionMessage>,
    blockchain: Arc<Mutex<Blockchain>>,
}

impl NodeMining {
    pub fn new(
               tx_blockchain: Sender<BlockchainMessage>,
               tx_transactions:Sender<TransactionMessage>,
               rx_transactions: Receiver<TransactionMessage>,
               blockchain: Arc<Mutex<Blockchain>>
    ) -> Self{
        NodeMining {
            tx_blockchain,
            tx_transactions,
            rx_transactions,
            blockchain,
        }
    }
    pub fn run(&mut self){
        loop{
            match self.rx_transactions.recv(){
                Ok(message) => {
                    match message {
                        TransactionMessage::TransactionVec(transactions) => self.mining(transactions),
                        (_)=>{}
                    }
                }
                Err(err) => {
                    warn!("Error with mining:{}", err)
                }
            }

            self.tx_transactions.send(GetTransaction()).unwrap();
        }
    }
    fn mining(&mut self, transactions: Vec<SerializedTransaction>){
        let last_block:Block;
        if let Ok(block) = self.blockchain.lock().expect("error with lock blockchain").get_last_block(){
            last_block = block;
        } else{
            self.blockchain.lock().expect("error with lock for create first").create_first_block();
            last_block = self.blockchain.lock().expect("error with lock blockchain").get_last_block().unwrap();
        }
        let mut nonce = 0;

        // let mut serialise_transaction = Vec::new();
        // for transaction in transactions{
        //     serialise_transaction.push(transaction.serialize());
        // }
        loop {
            {
                let blockchain_last_block = self.blockchain.lock().expect("error with lock blockchain").get_last_block().expect("something wrong with chain");
                if blockchain_last_block.get_id() != last_block.get_id() {
                    for transaction in transactions {
                        self.tx_transactions.send(AddTransaction(transaction)).unwrap()
                    }
                    break;
                }
            }

            let new_block = Block::new(last_block.get_id() + 1, transactions.clone(), last_block.get_hash(), nonce);
            if Blockchain::is_valid_block(&new_block) {
                self.tx_blockchain.send(BlockAdd(new_block)).unwrap();
            }
            nonce += 1;
        }
    }
}