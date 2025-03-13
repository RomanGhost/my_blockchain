use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender};

use chrono::{DateTime, Utc};

use crate::coin::node::blockchain::block::Block;
use crate::coin::node::blockchain::transaction::SerializedTransaction;
use crate::coin::node::node_blockchain::NodeBlockchain;
use crate::coin::node::node_message::{BlockchainMessage, TransactionMessage};
use crate::coin::server::server::Server;

pub struct AppState {
    server: Server,
    blockchain_tx: Sender<BlockchainMessage>,
    transaction_tx: Sender<TransactionMessage>,
    blockchain: Arc<Mutex<NodeBlockchain>>,

}

impl AppState {

    pub fn default() -> Self {
        AppState {
            server: Server::new(channel().0), // или используйте свой конструктор
            blockchain_tx: channel().0, // временные значения, будут перезаписаны в set_blockchain
            transaction_tx: channel().0,
            blockchain: Arc::new(Mutex::new(NodeBlockchain::new())),
        }
    }
    pub fn set_server(&mut self, server: Server) {
        self.server = server;
    }

    pub fn set_blockchain(&mut self,
                          blockchain_tx: Sender<BlockchainMessage>,
                          transaction_tx: Sender<TransactionMessage>,
                          blockchain: Arc<Mutex<NodeBlockchain>>
    ) {
        self.blockchain_tx = blockchain_tx;
        self.blockchain = blockchain;
        self.transaction_tx = transaction_tx;
    }

    pub fn add_block(&self, block:Block, is_force:bool){
        self.blockchain_tx.send(BlockchainMessage::BlockAdd(block)).unwrap();
    }

    pub fn check_chain(&self, chain:Vec<Block>){
        self.blockchain_tx.send(BlockchainMessage::ChainCheck(chain)).unwrap();
    }

    pub fn get_last_n_blocks(&self, n:usize)-> Vec<Block>{
        self.blockchain.lock().expect("Error lock blockchain node").get_last_n_blocks(n)
    }

    pub fn get_block_before(&self, date_time:DateTime<Utc>) -> Vec<Block>{
        self.blockchain.lock().expect("Error lock blockchain node").get_blocks_before(date_time)
    }

    pub fn add_transaction(&self, transaction:SerializedTransaction){
        self.transaction_tx.send(TransactionMessage::AddTransaction(transaction)).unwrap();
    }

    pub fn connect(&self, addr:String){
        self.server.connect(addr.as_str(), 7878).unwrap();
    }


}
