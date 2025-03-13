use std::sync::{Arc, atomic::{AtomicBool, Ordering}, Condvar, Mutex};
use std::sync::mpsc::channel;
use std::thread;

use log::{error, info, warn};
use sha2::digest::core_api::CoreWrapper;
use coin::app_state::AppState;
use crate::coin::node::{node_blockchain, node_mining};
use crate::coin::node::node_blockchain::NodeBlockchain;
use crate::coin::node::node_mining::NodeMining;
use crate::coin::node::node_transaction::NodeTransaction;
use crate::coin::server::pool;
use crate::coin::server::pool::connection_pool::ConnectionPool;
use crate::coin::server::protocol::p2p_protocol::P2PProtocol;
use crate::coin::server::server::Server;

mod coin;


fn initialize_server(mut app_state:AppState) -> (ConnectionPool, P2PProtocol, Server){
    let timeout = 12;

    let (pool_tx, pool_rx) = channel();
    let (protocol_tx, protocol_rx) = channel();

    let server = Server::new(pool_tx.clone());
    let app_state_server = Server::new(pool_tx.clone());
    app_state.set_server(app_state_server);

    let pool = ConnectionPool::new(timeout, pool_tx.clone(), pool_rx, protocol_tx.clone());
    let protocol = P2PProtocol::new(app_state, protocol_tx.clone(), protocol_rx, pool_tx.clone());

    (pool, protocol, server)
}

fn initialise_nodes(mut app_state: &mut AppState) -> (NodeTransaction, Arc<Mutex<NodeBlockchain>>, NodeMining){
    let(transaction_tx, transaction_rx) = channel();

    let node_transaction = NodeTransaction::new(transaction_tx);
    let node_blockchain = NodeBlockchain::new();

    let blockchain_tx = node_blockchain.get_sender();
    let blockchain = node_blockchain.get_blockchain();
    let transaction_tx = node_transaction.get_sender();

    let mutex_blockchain_node =  Arc::new(Mutex::new(node_blockchain));
    app_state.set_blockchain(blockchain_tx.clone(), transaction_tx.clone(), mutex_blockchain_node.clone());
    let node_mining = NodeMining::new(blockchain_tx, transaction_tx, transaction_rx, blockchain);

    (node_transaction, mutex_blockchain_node, node_mining)
}

fn main() {

    let mut app_state = AppState::default();

    let (nt, nb, nm) = initialise_nodes(&mut app_state);
    let (cp, p2p, server) = initialize_server(app_state);
    // let mut nt = NodeTransaction::new();

    std::env::set_var("RUST_LOG", "info");

    // // Инициализируем логгер
    env_logger::init();
    //
    // // Пример логгирования сообщений с разным уровнем
    info!("Program run");

}
