
use std::sync::{Arc, Mutex};
use std::io::Write;
use std::net::TcpStream;
use std::sync::mpsc::Sender;
use crate::coin::blockchain::block::Block;
use crate::coin::blockchain::transaction::Transaction;
use crate::coin::connection::ConnectionPool;
use crate::coin::message::r#type::Message;
use crate::coin::message::response;


pub struct P2PProtocol {
    connection_pool: Arc<Mutex<ConnectionPool>>,
    last_message_id:u64,
    sender:Sender<Message>,
}

impl P2PProtocol {
    pub fn new(connection_pool: Arc<Mutex<ConnectionPool>>, sender: Sender<Message>) -> Self {
        P2PProtocol {
            connection_pool,
            last_message_id:0,
            sender,
        }
    }

    pub fn handle_message(&mut self, message_json: &str) {
        let message_json = message_json.trim_end_matches('\0');

        match Message::from_json(message_json) {
            Ok(message) => {
                let message_id = message.get_id();

                if message_id < self.last_message_id{
                    return
                }else{
                    self.last_message_id = message_id;
                }

                self.sender.send(message.clone()).unwrap();
                self.broadcast(message, true);
            }
            Err(e) => {
                eprintln!("Failed to deserialize response_message: {}", e);
            }
        }
    }

    fn handle_ping(&self, peer_address: &str, stream: &mut TcpStream) {
        println!("Handling ping from: {}", peer_address);
        let response = format!("pong from {}", peer_address);
        stream.write_all(response.as_bytes()).unwrap();
    }

    pub fn handle_text(&mut self, message: String, receive:bool) {
        let response_message = response::TextMessage::new(message);
        let response_message = Message::TextMessage(response_message);

        self.broadcast(response_message, receive);
    }

    pub fn handle_block(&mut self, block: Block, force:bool, receive:bool) {
        let response_message = response::BlockMessage::new(block, force);
        let response_message = Message::BlockMessage(response_message);

        self.broadcast(response_message, receive);
    }
    fn handle_transaction(&mut self, message: Transaction, receive:bool) {
        let response_message = response::TransactionMessage::new(message);
        let response_message = Message::TransactionMessage(response_message);

        self.broadcast(response_message, receive);
    }

    fn handle_peers(&self, stream: &mut TcpStream) {
        let connection_pool = self.connection_pool.lock().unwrap();
        let peer_addresses = connection_pool.get_peer_addresses();
        let peers_list = peer_addresses.join(", ");
        let response = format!("Peers: {}", peers_list);
        stream.write_all(response.as_bytes()).unwrap();
    }

    pub fn broadcast(&mut self, mut message:Message, receive:bool){
        message.set_id(self.last_message_id);
        if !receive {
            self.last_message_id += 1;
        }

        let mut connection_pool = self.connection_pool.lock().unwrap();
        let serialize_message = message.to_json();

        connection_pool.broadcast(serialize_message.as_ref());
    }
}
