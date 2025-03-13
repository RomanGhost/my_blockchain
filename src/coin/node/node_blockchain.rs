use std::fmt::Pointer;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Receiver, RecvError, Sender};
use std::thread;
use std::thread::Thread;
use std::time::Duration;

use chrono::{DateTime, Utc};
use log::{error, warn};

use crate::coin::node::blockchain::block::Block;
use crate::coin::node::blockchain::blockchain::Blockchain;
use crate::coin::node::blockchain::transaction::Transaction;
use crate::coin::node::node_message::{BlockchainMessage, TransactionMessage};
use crate::coin::node::node_message::BlockchainMessage::BlockAdd;
use crate::coin::node::node_message::TransactionMessage::{AddTransaction, GetTransaction};

pub struct NodeBlockchain{
    blockchain: Arc<Mutex<Blockchain>>,
    tx: Sender<BlockchainMessage>,
    rx: Receiver<BlockchainMessage>,
}

impl NodeBlockchain{
    pub fn new() -> Self{
        let (tx, rx) = channel();
        let blockchain = Arc::new(Mutex::new(Blockchain::new()));
        NodeBlockchain{
            blockchain,
            tx, rx,
        }
    }

    pub fn run(&mut self) {

        loop {
            match self.rx.recv_timeout(Duration::from_secs(1)) {
                Ok(message) => {

                },
                Err(err) => {
                    error!("Unknown blockchain message type: {}", err);
                }
            }
        }
    }

    pub fn get_last_n_blocks(&self, n:usize) -> Vec<Block>{
        self.blockchain.lock().expect("blocking blockchain for get chain").get_last_n_blocks(n)
    }

    pub fn get_blocks_before(&self, date_time:DateTime<Utc>) -> Vec<Block>{
        self.blockchain.lock().expect("blocking blockchain for get chain").get_blocks_before(date_time)
    }

    pub fn get_blockchain(&self) -> Arc<Mutex<Blockchain>> {
        self.blockchain.clone()
    }

    pub fn get_sender(&self) -> Sender<BlockchainMessage> {
        return self.tx.clone();
    }
}
