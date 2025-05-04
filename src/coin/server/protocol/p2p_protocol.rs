use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;
use chrono::{TimeZone, Utc};
use log::{debug, error, info, warn};

use crate::coin::app_state::AppState;
use crate::coin::node::blockchain::block::Block;
use crate::coin::server::pool::pool_message::PoolMessage;
use crate::coin::server::pool::pool_message::PoolMessage::BroadcastMessage;
use crate::coin::server::protocol::message::response;
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
                        _ =>{
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
            _ => ()
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
            _ => {
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
        let chain = self.app_state.get_block_before(datetime.timestamp()); //TODO обработка ошибки
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::{channel, RecvTimeoutError};
    use std::time::Duration;
    use chrono::{Utc, TimeZone};

    use crate::coin::app_state::AppState;
    use crate::coin::server::pool::pool_message::PoolMessage::BroadcastMessage;
    use crate::coin::server::protocol::message::r#type::Message;
    use crate::coin::server::protocol::message::request::{LastNBlocksMessage, BlocksBeforeMessage};
    use crate::coin::server::protocol::message::response::{MessageAnswerFirstInfo, TextMessage};

    /// Вспомогалка: создаёт P2PProtocol с пустым AppState и новыми каналами.
    fn make_protocol() -> (P2PProtocol, std::sync::mpsc::Receiver<PoolMessage>) {
        let app_state = AppState::default();          // убедитесь, что AppState::new() есть
        let (tx_proto, rx_proto) = channel();
        let (tx_pool, rx_pool) = channel();
        let proto = P2PProtocol::new(app_state, tx_proto, rx_proto, tx_pool);
        (proto, rx_pool)
    }

    #[test]
    fn test_get_sender_protocol() {
        let (mut proto, _) = make_protocol();
        let sender = proto.get_sender_protocol();

        let test_msg = Message::RawMessage("foo".into());
        sender.send(test_msg.clone()).unwrap();
        // Проверяем, что proto.rx получил сообщение
        assert_eq!(proto.rx.recv_timeout(Duration::from_millis(10)).unwrap().to_json(), test_msg.to_json());
    }

    #[test]
    fn test_send_message() {
        let (mut proto, rx_pool) = make_protocol();
        assert_eq!(proto.last_message_id, 0);

        let msg = Message::ResponseTextMessage(TextMessage::new("hello".to_string()));
        proto.send_message(msg);

        let got = rx_pool.recv_timeout(Duration::from_secs(1)).unwrap();
        if let BroadcastMessage(json) = got {
            // id должно быть 1 и присутствовать payload
            assert!(json.contains("\"id\":1"));
            assert!(json.contains("hello"));
        } else {
            panic!("Ожидали BroadcastMessage, получили {:?}", got);
        }
        assert_eq!(proto.last_message_id, 1);
    }

    #[test]
    fn test_process_request_message_info() {
        let (mut proto, rx_pool) = make_protocol();
        // отправляем запрос первой информации
        let mut req = Message::RequestMessageInfo(request::MessageFirstInfo::new());
        // process_message ждёт, что req.id не важен, он не сравнивается
        proto.process_message(req);

        // Должно уйти два BroadcastMessage:
        // 1) ответ на RequestMessageInfo (ResponseMessageInfo)
        // 2) цепочка (ResponseChainMessage)
        let first = rx_pool.recv_timeout(Duration::from_secs(1)).unwrap();
        let second = rx_pool.recv_timeout(Duration::from_secs(1)).unwrap();

        match first {
            BroadcastMessage(json) => {
                assert!(json.contains("ResponseMessageInfo"));
                assert!(json.contains("\"id\":1"));
            }
            _ => panic!("1-е сообщение: ожидали ResponseMessageInfo"),
        }
        match second {
            BroadcastMessage(json) => {
                assert!(json.contains("ResponseChainMessage"));
                // id==2
                assert!(json.contains("\"id\":2"));
            }
            _ => panic!("2-е сообщение: ожидали ResponseChainMessage"),
        }
    }

    #[test]
    fn test_process_response_message_info_updates_last_id() {
        let (mut proto, rx_pool) = make_protocol();
        // сначала поднимем last_message_id до 5
        proto.last_message_id = 5;

        // отправляем ResponseMessageInfo с id = 10
        let mut ans = Message::ResponseMessageInfo(MessageAnswerFirstInfo::new());
        ans.set_id(10);
        proto.process_message(ans);

        // pool_tx не должен ничего получить (мы только обновляем last_message_id)
        assert!(rx_pool.recv_timeout(Duration::from_millis(50))
            .err()
            .map(|e| matches!(e, RecvTimeoutError::Timeout))
            .unwrap());
        assert_eq!(proto.last_message_id, 10);
    }

    #[test]
    fn test_process_message_ignores_old() {
        let (mut proto, rx_pool) = make_protocol();
        proto.last_message_id = 5;

        let mut old = Message::ResponseTextMessage(TextMessage::new("old".to_string()));
        old.set_id(3);

        proto.process_message(old);
        // Ничего не приходит
        assert!(rx_pool.recv_timeout(Duration::from_millis(50))
            .err()
            .map(|e| matches!(e, RecvTimeoutError::Timeout))
            .unwrap());
        assert_eq!(proto.last_message_id, 5);
    }

    #[test]
    fn test_process_message_broadcasts_new_text() {
        let (mut proto, rx_pool) = make_protocol();
        proto.last_message_id = 1;

        let mut txt = Message::ResponseTextMessage(TextMessage::new("world".to_string()));
        txt.set_id(10);
        proto.process_message(txt);

        let got = rx_pool.recv_timeout(Duration::from_secs(1)).unwrap();
        if let BroadcastMessage(json) = got {
            assert!(json.contains("\"id\":10"));
            assert!(json.contains("world"));
        } else {
            panic!("Ожидали BroadcastMessage");
        }
        assert_eq!(proto.last_message_id, 10);
    }

    #[test]
    fn test_process_request_last_n_blocks() {
        let (mut proto, rx_pool) = make_protocol();
        proto.last_message_id = 0;

        // создаём запрос последних 3 блоков
        let mut req = LastNBlocksMessage::new(3);
        req.set_id(7);
        proto.process_message(Message::RequestLastNBlocksMessage(req));

        // 1) эхо самого запроса
        let echo = rx_pool.recv_timeout(Duration::from_secs(1)).unwrap();
        // 2) отправка цепочки
        let chain = rx_pool.recv_timeout(Duration::from_secs(1)).unwrap();

        if let BroadcastMessage(json) = echo {
            assert!(json.contains("RequestLastNBlocksMessage"));
            assert!(json.contains("\"id\":7"));
        } else { panic!("ожидали эхо BroadcastMessage"); }

        if let BroadcastMessage(json) = chain {
            assert!(json.contains("ResponseChainMessage"));
            // id должно стать 8
            assert!(json.contains("\"id\":8"));
        } else { panic!("ожидали BroadcastMessage с цепочкой"); }

        assert_eq!(proto.last_message_id, 8);
    }

    #[test]
    fn test_process_request_blocks_before() {
        let (mut proto, rx_pool) = make_protocol();
        proto.last_message_id = 0;

        // запрос блоков до UNIX-времени 1_600_000_000
        let mut req = BlocksBeforeMessage::new(1_600_000_000);
        req.set_id(5);
        proto.process_message(Message::RequestBlocksBeforeMessage(req));

        // 1) эхо запроса
        let echo = rx_pool.recv_timeout(Duration::from_secs(1)).unwrap();
        // 2) цепочка
        let chain = rx_pool.recv_timeout(Duration::from_secs(1)).unwrap();

        if let BroadcastMessage(json) = echo {
            assert!(json.contains("RequestBlocksBeforeMessage"));
            assert!(json.contains("\"id\":5"));
        } else { panic!("ожидали эхо BroadcastMessage"); }

        if let BroadcastMessage(json) = chain {
            assert!(json.contains("ResponseChainMessage"));
            assert!(json.contains("\"id\":6"));
        } else { panic!("ожидали BroadcastMessage с цепочкой"); }

        assert_eq!(proto.last_message_id, 6);
    }
}
