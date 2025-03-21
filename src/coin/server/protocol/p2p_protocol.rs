use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;
use chrono::{TimeZone, Utc};
use log::{debug, error, info, warn};

use crate::coin::app_state::AppState;
use crate::coin::node::blockchain::block::Block;
use crate::coin::server::pool::pool_message::PoolMessage;
use crate::coin::server::pool::pool_message::PoolMessage::BroadcastMessage;
use crate::coin::server::protocol::message::{request, response};
use crate::coin::server::protocol::message::r#type::Message;
use crate::coin::server::protocol::message::request::{BlocksBeforeMessage, LastNBlocksMessage};
use crate::coin::server::protocol::message::response::{BlockMessage, ChainMessage, PeerMessage, TransactionMessage};

pub struct P2PProtocol{
    //Каналы для коммуникации с потоком протокола
    tx: Sender<Message>,
    rx: Receiver<Message>,

    // Каналы для коммуникации с peer
    pool_tx: Sender<PoolMessage>,

    last_message_id: u64,
    app_state: AppState,
}

impl P2PProtocol{
    pub fn new(app_state: AppState, tx:Sender<Message>, rx:Receiver<Message>, pool_tx:Sender<PoolMessage>) -> Self{
        P2PProtocol{
            tx, rx, pool_tx,
            last_message_id: 0,
            app_state,
        }
    }

    pub fn get_sender_protocol(&self) -> Sender<Message> {
        self.tx.clone()
    }

    pub fn run(&mut self){
        loop {
            match self.rx.recv_timeout(Duration::from_secs(1)) {
                // input from other nodes
                Ok(Message::RawMessage(message_json)) => {
                    let message_json = message_json.trim_end_matches('\0');
                    debug!("P2P protocol: message json: {}", message_json);
                    match Message::from_json(message_json) {
                        Ok(message) => {
                            self.process_message(message);
                        },
                        Err(e) => {
                            warn!("Failed to deserialize response_message: {}, {}", e, message_json);
                        }
                    };
                },
                // input from this server
                Ok(message) => {
                    self.send_message(message);
                },
                Err(err) => {
                    match err {
                        RecvTimeoutError => {
                            continue
                        }
                        (_) =>{
                            error!("Unknown peer message type: {}", err);
                        }
                    }
                }
            }
        }
    }

    /// Входящие сообщения
    fn process_message(&mut self, message: Message){
        match message{
            Message::RequestMessageInfo(_) => {
                info!("Type:RequestMessageInfo get");
                self.send_first_message();

                return
            }
            Message::ResponseMessageInfo(msg) => {
                info!("Type:ResponseMessageInfo get");
                let message_id = msg.get_id();
                if self.last_message_id < message_id {
                    self.last_message_id = message_id;
                }
                debug!("Получено сообщение об id сообщения: {}/{}", msg.get_id(), self.last_message_id);
                return
            }
            (_) => ()
        }

        let message_id = message.get_id();
        //Если текущее сообщение меньше чем сообщение чата
        if message_id <= self.last_message_id{
            debug!("Message less main ID: {}<{}", message_id, self.last_message_id);
            return;
        }else{
            self.last_message_id = message_id;
        }
        self.pool_tx.send(PoolMessage::BroadcastMessage(message.to_json())).expect("TODO: panic message");

        match message {
            Message::ResponseTransactionMessage(msg) =>self.process_transaction(msg),
            Message::ResponseBlockMessage(msg )=>self.process_block(msg),
            Message::ResponseChainMessage(msg)=>self.process_chain(msg),
            Message::ResponsePeerMessage(msg)=>self.process_peer(msg),

            Message::RequestLastNBlocksMessage(msg) => self.send_last_n_locks(msg),
            Message::RequestBlocksBeforeMessage(msg) => self.send_block_before(msg),
            Message::ResponseTextMessage(msg) => {
                info!("Get text message: {}", msg.get_text());
            },
            (_) => {
                warn!("Unknown type message");
            }
        }
    }

    fn send_message(&mut self, message: Message) {
        self.last_message_id += 1;

        let mut send_message = message;
        send_message.set_id(self.last_message_id);
        let json_message = send_message.to_json();

        let broadcast_message = BroadcastMessage(json_message);
        self.pool_tx.send(broadcast_message).unwrap();
    }

    fn process_block(&self, msg:BlockMessage) {
        let is_force_block = msg.is_force();
        let new_block = msg.get_block();
        debug!("Get new block: {}", new_block.get_id());

        self.app_state.add_block(new_block, is_force_block); //TODO(обработать ошибки)
    }

    fn process_transaction(&self, msg:TransactionMessage) {
        let new_transaction = msg.get_transaction();
        debug!("Get new transaction");
        self.app_state.add_transaction(new_transaction)
    }

    fn process_chain(&self, msg:ChainMessage) {
        let chain = msg.get_chain();
        debug!("Get new transaction");
        self.app_state.check_chain(chain); //TODO(обработать ошибки)
    }

    fn process_peer(&self, msg:PeerMessage){
        let peer = msg.get_peer();
        info!("New peer");
        self.app_state.connect(peer);
    }

    fn send_first_message(&mut self){
        self.last_message_id += 1;
        let mut message_info = response::MessageAnswerFirstInfo::new();
        message_info.set_id(self.last_message_id);
        let response_message = Message::ResponseMessageInfo(message_info);

        let json_message = response_message.to_json();
        let broadcast_message = BroadcastMessage(json_message);
        self.pool_tx.send(broadcast_message).unwrap();

        let chain = self.app_state.get_from_first_block();//TODO обработка ошибки
        self.send_chain(chain)
    }

    fn send_last_n_locks(&mut self, msg:LastNBlocksMessage){
        let n = msg.get_n();
        debug!("Request chain blocks");
        let chain = self.app_state.get_last_n_blocks(n); //TODO Обработка ошибки
        self.send_chain(chain);
    }

    fn send_block_before(&mut self, msg:BlocksBeforeMessage){
        let date_time_unix = msg.get_time();
        let datetime = Utc.timestamp_opt(date_time_unix, 0).unwrap();
        debug!("Get block before");
        let chain = self.app_state.get_block_before(datetime); //TODO обработка ошибки
        self.send_chain(chain);
    }

    fn send_chain(&mut self, chain:Vec<Block>){
        self.last_message_id += 1;
        let mut chain_message = ChainMessage::new(chain);
        chain_message.set_id(self.last_message_id);

        let json_message = Message::ResponseChainMessage(chain_message).to_json();
        let broadcast_message = BroadcastMessage(json_message);
        self.pool_tx.send(broadcast_message).unwrap();
    }
}