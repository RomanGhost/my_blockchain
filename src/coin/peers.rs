
use std::sync::{Arc, Mutex};
use std::io::Write;
use std::net::TcpStream;
use std::sync::mpsc::Sender;
use crate::coin::connection::ConnectionPool;
use crate::coin::message::message::{BlockMessage, Message, TextMessage, TransactionMessage};

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
                    return;
                }
                else{
                    self.last_message_id = message_id;
                }

                self.broadcast(message);
                // Обрабатываем разные варианты сообщений
                // match message_.get_type() {
                //     Message::BlockMessage(block_msg) => {
                //         dbg!("Received BlockMessage with id: {}", block_msg.get_id());
                //         // Здесь можно добавить логику для работы с BlockMessage
                //     }
                //     Message::TransactionMessage(tx_msg) => {
                //         dbg!("Received TransactionMessage with id: {}", tx_msg.get_id());
                //         // Здесь можно добавить логику для работы с TransactionMessage
                //     }
                //     Message::TextMessage(text_msg) => {
                //         self.handle_text(text_msg);
                //         // Здесь можно добавить логику для работы с TextMessage
                //     }
                // }
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

    fn handle_text(&mut self, message: TextMessage) {
        let new_message = Message::TextMessage(message);
        self.sender.send(new_message.clone()).unwrap();

        self.broadcast(new_message);
    }

    fn handle_block(&mut self, message: BlockMessage) {
        println!("Handling block: {}", message.get_id());
        let new_message = Message::BlockMessage(message);

        self.broadcast(new_message);
    }

    fn handle_force_block(&mut self, message: BlockMessage) {
        println!("Handling block: {}", message.get_id());
        let new_message = Message::BlockMessage(message);

        self.broadcast(new_message);
    }

    fn handle_transaction(&mut self, message: TransactionMessage) {
        println!("Handling transaction: {}", message.get_id());
        let new_message = Message::TransactionMessage(message);

        self.broadcast(new_message);
    }

    fn handle_peers(&self, stream: &mut TcpStream) {
        let connection_pool = self.connection_pool.lock().unwrap();
        let peer_addresses = connection_pool.get_peer_addresses();
        let peers_list = peer_addresses.join(", ");
        let response = format!("Peers: {}", peers_list);
        stream.write_all(response.as_bytes()).unwrap();
    }

    pub fn broadcast(&mut self, mut message:Message){
        self.sender.send(message.clone()).unwrap();
        message.set_id(self.last_message_id);

        let mut connection_pool = self.connection_pool.lock().unwrap();
        let serialize_message = message.to_json();
        self.last_message_id += 1;

        connection_pool.broadcast(serialize_message.as_ref());
    }
}
