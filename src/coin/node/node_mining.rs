use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;
use log::{debug, error, warn};

use crate::coin::node::blockchain::block::Block;
use crate::coin::node::blockchain::blockchain::Blockchain;
use crate::coin::node::blockchain::transaction::SerializedTransaction;
use crate::coin::node::node_message::TransactionMessage;
use crate::coin::node::node_message::TransactionMessage::{AddTransaction, GetTransaction};

pub struct NodeMining {
    tx_transactions:Sender<TransactionMessage>,
    rx_transactions: Receiver<TransactionMessage>,
    tx_external: Sender<Block>,
    blockchain: Arc<Mutex<Blockchain>>,
}

impl NodeMining {
    pub fn new(
               tx_transactions:Sender<TransactionMessage>,
               rx_transactions: Receiver<TransactionMessage>,
               tx_external: Sender<Block>,
               blockchain: Arc<Mutex<Blockchain>>
    ) -> Self{
        NodeMining {
            tx_transactions,
            rx_transactions,
            tx_external,
            blockchain,
        }
    }
    pub fn run(&mut self){
        loop{
            match self.rx_transactions.recv_timeout(Duration::from_millis(3000)){
                Ok(message) => {
                    match message {
                        TransactionMessage::TransactionVec(transactions) => self.mining(transactions),
                        _=>{debug!("transactionInfo");}
                    }
                }
                Err(err) => {
                    match err {
                        RecvTimeoutError=> {
                            self.tx_transactions.send(GetTransaction()).unwrap();
                            debug!("Request transactions in mining");
                        }
                        _ => {
                            error!("Mining | Unknown message type: {}", err);
                        }
                    }
                }
            }
        }
    }

    fn mining(&mut self, transactions: Vec<SerializedTransaction>) {
        debug!("Mining new block!");

        // Захватываем блокировку один раз для получения последнего блока.
        let last_block = {
            let mut blockchain = self.blockchain.lock().unwrap_or_else(|e| {
                error!("Mutex poisoned: {}", e);
                panic!("Critical error with blockchain lock")
            });

            match blockchain.get_last_block() {
                Ok(block) => block.clone(),
                Err(_) => {
                    warn!("Creating new chain, generating first block");
                    blockchain.create_first_block();
                    blockchain.get_last_block().expect("Newly created block should exist")
                }
            }
        };

        let mut nonce = 0;

        loop {
            debug!("wait of lock");
            // Захватываем блокировку на каждой итерации
            let blockchain_result = self.blockchain.lock();
            if blockchain_result.is_err() {
                error!("Mutex poisoned during mining iteration");
                break;
            }
            let mut blockchain = blockchain_result.unwrap();
            let current_last_block = match blockchain.get_last_block() {
                Ok(block) => block,
                Err(_) => {
                    error!("Corrupted blockchain state");
                    drop(blockchain);
                    break;
                }
            };

            // Если последний блок изменился, значит другой поток уже обновил цепочку
            if current_last_block.get_id() != last_block.get_id() {
                drop(blockchain); // Освобождаем блокировку
                // Отправляем транзакции обратно
                for transaction in transactions.clone() {
                    if let Err(e) = self.tx_transactions.send(AddTransaction(transaction)) {
                        error!("Failed to send transaction: {}", e);
                    }
                }
                break;
            }

            // Создаем новый блок с текущим nonce
            let new_block = Block::new(
                current_last_block.get_id() + 1,
                transactions.clone(),
                current_last_block.get_hash(),
                nonce,
            );

            if Blockchain::is_valid_block(&new_block) {
                debug!("New block found with nonce: {}", nonce);
                // Пытаемся добавить блок
                if let Err(e) = blockchain.add_block(new_block.clone()) {
                    error!("Failed to add valid block: {}", e);
                } else {
                    match self.tx_external.send(new_block)
                    {
                        Ok(()) => { /* всё ок */ }
                        Err(e) => {
                            error!("Не удалось отправить BlockMessage: {}. Завершаем поток.", e);
                            break; // или return, чтобы выйти из потока
                        }
                    }

                }
                drop(blockchain);
                break;
            } else {
                nonce += 1;
            }
            debug!("Nonce: {}", nonce);

            // Явно освобождаем блокировку перед следующей итерацией
            drop(blockchain);
            // Опционально: сделать паузу для снижения нагрузки на CPU
            // std::thread::sleep(Duration::from_millis(100));
        }
    }


    pub fn get_blockchain(&self)-> Arc<Mutex<Blockchain>> {
        self.blockchain.clone()
    }
}