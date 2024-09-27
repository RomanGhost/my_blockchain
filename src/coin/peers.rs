
use std::sync::{Arc, Mutex};
use std::io::Write;
use std::net::TcpStream;
use std::sync::mpsc::Sender;
use crate::coin::blockchain::block::Block;
use crate::coin::blockchain::transaction::Transaction;
use crate::coin::connection::ConnectionPool;
use crate::coin::message::r#type::Message;
use crate::coin::message::{request, response};



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

                match message {
                    Message::RequestMessageInfo(_) => {
                        self.response_first_message();
                        return;
                    }

                    Message::ResponseMessageInfo(msg) => {
                        let message_id = msg.get_id();
                        if self.last_message_id < message_id {
                            self.last_message_id = message_id;
                        }
                        println!("Получено сообщение об id сообщения: {}/{}", msg.get_id(), self.last_message_id);
                        return;
                    }
                    (_)=>{}
                }

                let message_id = message.get_id();
                //Если текущее сообщение меньше чем сообщение чата
                if message_id <= self.last_message_id{
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

    fn response_ping(&self, peer_address: &str, stream: &mut TcpStream) {
        println!("Handling ping from: {}", peer_address);
        let response = format!("pong from {}", peer_address);
        stream.write_all(response.as_bytes()).unwrap();
    }

    pub fn request_first_message(&mut self){
        println!("Отправлено сообщение на запрос id  сообщения в чате");
        let response_message = request::MessageFirstInfo::new();
        let response_message = Message::RequestMessageInfo(response_message);

        self.broadcast(response_message, true);
    }

    pub fn response_first_message(&mut self){
        let response_message = response::MessageAnswerFirstInfo::new();
        let response_message = Message::ResponseMessageInfo(response_message);

        self.broadcast(response_message, true);
    }

    pub fn response_text(&mut self, message: String, receive:bool) {
        let response_message = response::TextMessage::new(message);
        let response_message = Message::ResponseTextMessage(response_message);

        self.broadcast(response_message, receive);
    }

    pub fn response_block(&mut self, block: Block, force:bool, receive:bool) {
        let response_message = response::BlockMessage::new(block, force);
        let response_message = Message::ResponseBlockMessage(response_message);

        self.broadcast(response_message, receive);
    }
    fn response_transaction(&mut self, message: Transaction, receive:bool) {
        let response_message = response::TransactionMessage::new(message);
        let response_message = Message::ResponseTransactionMessage(response_message);

        self.broadcast(response_message, receive);
    }

    fn response_peers(&self, stream: &mut TcpStream) {
        let connection_pool = self.connection_pool.lock().unwrap();
        let peer_addresses = connection_pool.get_peer_addresses();
        let peers_list = peer_addresses.join(", ");
        let response = format!("Peers: {}", peers_list);
        stream.write_all(response.as_bytes()).unwrap();
    }

    pub fn broadcast(&mut self, mut message:Message, receive:bool){
        if !receive {
            self.last_message_id += 1;
        }
        message.set_id(self.last_message_id);

        let serialize_message = message.to_json();
        let mut connection_pool = self.connection_pool.lock().unwrap();

        connection_pool.broadcast(serialize_message.as_ref());
    }
}
