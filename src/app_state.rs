use std::sync::mpsc::{Receiver, Sender};
use crate::coin::blockchain::block::Block;
use crate::coin::server::protocol::message::r#type::Message;
use crate::coin::server::server::Server;

pub struct AppState {
    pub protocol_tx: Sender<Message>,
    blockchain_rx: Receiver<Block>,
    blockchain_tx: Sender<Block>,

}

impl AppState {
    pub fn add_block(&self, block:Block, is_force:bool){
        self.blockchain_tx.send(block).unwrap()
    }
}
