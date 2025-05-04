use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender};

use chrono::{DateTime, Utc};
use log::{debug, info};
use crate::coin::db::BlockDatabase;
use crate::coin::node::blockchain::block::Block;
use crate::coin::node::blockchain::blockchain::{Blockchain, validate_chain};
use crate::coin::node::blockchain::transaction::SerializedTransaction;
use crate::coin::node::node_message::TransactionMessage;
use crate::coin::server::server::Server;

pub struct AppState {
    database: Arc<Mutex<BlockDatabase>>,
    server: Server,
    transaction_tx: Sender<TransactionMessage>,
    blockchain: Arc<Mutex<Blockchain>>,
}

impl AppState {

    pub fn new(databaseMutex:  Arc<Mutex<BlockDatabase>>) -> Self {
        AppState {
            server: Server::new(channel().0),
            transaction_tx: channel().0,
            blockchain: Arc::new(Mutex::new(Blockchain::new())),
            database: databaseMutex
        }
    }
    pub fn set_server(&mut self, server: Server) {
        self.server = server;
    }

    pub fn set_blockchain(&mut self,
                          transaction_tx: Sender<TransactionMessage>,
                          blockchain: Arc<Mutex<Blockchain>>
    ) {
        self.blockchain = blockchain;
        self.transaction_tx = transaction_tx;
    }

    pub fn insert_block_into_db(&self, block:&Block) -> rusqlite::Result<()> {
        info!("Insert block into DB");
        self.database.lock().expect("Can't lock mutex of DB").insert_block(block)
    }
    pub fn add_block(&self, block:Block, is_force:bool){
        self.insert_block_into_db(&block);
        if is_force{
            self.blockchain.lock().unwrap().add_force_block(block);
        }else {
            self.blockchain.lock().unwrap().add_block(block).expect("Error add block to chain");
        }
    }

    pub fn check_chain(&self, chain:Vec<Block>){
        validate_chain(&chain);
    }

    pub fn get_from_first_block(&self) -> Vec<Block> {
        self.blockchain.lock().expect("Error lock blockchain node").get_full_chain()
    }

    pub fn get_last_n_blocks(&self, n:usize)-> Vec<Block>{
        self.blockchain.lock().expect("Error lock blockchain node").get_last_n_blocks(n)
    }

    pub fn get_block_before(&self, date_time:i64) -> Vec<Block>{
        self.blockchain.lock().expect("Error lock blockchain node").get_blocks_before(date_time)
    }

    pub fn add_transaction(&self, transaction:SerializedTransaction){
        self.transaction_tx.send(TransactionMessage::AddTransaction(transaction)).expect("send transaction to channel");
    }

    pub fn connect(&self, addr:String){
        debug!("send request to server for connect: {}", addr);
        self.server.connect(format!("{}:7878", addr)).unwrap();
    }
}
