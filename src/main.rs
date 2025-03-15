use std::sync::{Arc, atomic::{AtomicBool, Ordering}, Condvar, Mutex};
use std::sync::mpsc::{channel, Sender};
use std::{io, thread};
use std::io::Write;
use std::time::Duration;
use log::{error, info, warn};
use sha2::digest::core_api::CoreWrapper;
use coin::app_state::AppState;
use crate::coin::node::blockchain::block::Block;
use crate::coin::node::blockchain::blockchain::Blockchain;
use crate::coin::node::node_mining::NodeMining;
use crate::coin::node::node_transaction::NodeTransaction;
use crate::coin::server::pool;
use crate::coin::server::pool::connection_pool::ConnectionPool;
use crate::coin::server::protocol::message::r#type::Message;
use crate::coin::server::protocol::message::response::{BlockMessage, TextMessage};
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

fn initialise_nodes(mut app_state: &mut AppState, tx_external: Sender<Block>,) -> (NodeTransaction, NodeMining){
    let(transaction_tx, transaction_rx) = channel();

    let node_transaction = NodeTransaction::new(transaction_tx);

    let mutex_blockchain = Arc::new(Mutex::new(Blockchain::new()));
    let transaction_tx = node_transaction.get_sender();

    app_state.set_blockchain(transaction_tx.clone(), mutex_blockchain.clone());
    let node_mining = NodeMining::new(transaction_tx, transaction_rx, tx_external, mutex_blockchain);

    (node_transaction, node_mining)
}

fn get_input_text(info_text: &str) -> String {
    print!("{}: ", info_text);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => input.trim().to_string(),
        Err(e) => {
            eprintln!("Error reading input: {}", e);
            String::new()
        }
    }
}

fn command_input(protocol_sender: Sender<Message>){
    loop {
        println!("\nДоступные команды:");
        println!("1. Подключиться к другому серверу (connect <IP>:<port>)");
        println!("2. Вещать сообщение всем пирами (broadcast <сообщение>)");
        println!("3. Выйти (exit)");

        match get_input_text("Введите команду").split_whitespace().collect::<Vec<&str>>().as_slice() {
            ["broadcast", message @ ..] if !message.is_empty() => {
                let response_message = Message::ResponseTextMessage(TextMessage::new(message.join(" ")));
                protocol_sender.send(response_message).unwrap()
            }
            ["exit"] => {
                println!("Выход из программы.");
                break;
            },
            _ => println!("Неверная команда."),
        }
    }
}

fn main() {
    std::env::set_var("RUST_LOG", "debug");

    // // Инициализируем логгер
    env_logger::init();
    //
    // // Пример логгирования сообщений с разным уровнем
    info!("Program run");

    let mut app_state = AppState::default();

    let (tx, rx) = channel();
    let (mut nt, mut nm) = initialise_nodes(&mut app_state, tx);
    let (mut cp, mut p2p, mut server) = initialize_server(app_state);

    let protocol_sender = p2p.get_sender_protocol();
    let protocol_sender_thread = p2p.get_sender_protocol();

    let node_transaction_thread = thread::spawn(move || {
        nt.run();
    });
    let node_mining_thread = thread::spawn(move || {
        nm.run();
    });
    let block_to_message_thread = thread::spawn(move || {
        for b in rx.recv(){
            protocol_sender_thread.send(Message::ResponseBlockMessage(BlockMessage::new(b, false))).unwrap()
        }
    });

    let connection_pool_thread = thread::spawn(move || {
        cp.run();
    });
    let protocol_thread = thread::spawn(move || {
        p2p.run();
    });


    let is_container = true;
    //UserNode
    if !is_container {
        let server_copy = Server::new(server.get_pool_sender());
        let server_thread = thread::spawn(move || {
            server.run("0.0.0.0:7878").expect("Can't run server thread");
        });

        let server = server_copy;

        server.connect("localhost", 7879).expect("Connect to ");
        //UserNode
        command_input(protocol_sender);
        server_thread.join().unwrap();
    } else {
        match std::env::var("ConnectAddr") {
            Ok(val) => server.connect(val.as_str(), 7878).unwrap(),
            Err(err) => info!("Error read env: {}", err)
        }
        server.run("0.0.0.0:7878").expect("Can't run server thread");
    }
    protocol_thread.join().unwrap();
    connection_pool_thread.join().unwrap();
}
