use std::sync::{Arc, atomic::{AtomicBool, Ordering}, Condvar, Mutex};
use std::sync::mpsc::channel;
use std::thread;
use log::{info, warn, error};
use crate::app_state::AppState;
use crate::coin::blockchain::blockchain::Blockchain;
use crate::coin::server::server::Server;
use crate::coin::server::pool;
use crate::coin::server::pool::connection_pool::ConnectionPool;
use crate::coin::server::protocol::p2p_protocol::P2PProtocol;

mod coin;
mod app_state;


fn initialize_server(mut app_state:AppState) -> (ConnectionPool, P2PProtocol, Server){
    let timeout = 12;

    let (pool_tx, pool_rx) = channel();
    let (protocol_tx, protocol_rx) = channel();

    app_state.protocol_tx = protocol_tx.clone();

    let pool = ConnectionPool::new(timeout, pool_tx.clone(), pool_rx, protocol_tx.clone());
    let protocol = P2PProtocol::new(app_state, protocol_tx.clone(), protocol_rx, pool_tx.clone());
    let server = Server::new(pool_tx);

    (pool, protocol, server)

}

fn main() {
    std::env::set_var("RUST_LOG", "info");

    // // Инициализируем логгер
    env_logger::init();
    //
    // // Пример логгирования сообщений с разным уровнем
    info!("Program run");



}
