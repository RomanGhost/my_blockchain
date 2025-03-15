use std::collections::BinaryHeap;
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::time::Duration;

use log::error;

use crate::coin::node::blockchain::block::Block;
use crate::coin::node::blockchain::blockchain::Blockchain;
use crate::coin::node::blockchain::transaction::{SerializedTransaction, Transaction};
use crate::coin::node::node_message::TransactionMessage;

pub struct NodeTransaction{
    transaction_queue: BinaryHeap<SerializedTransaction>,
    tx: Sender<TransactionMessage>,
    rx: Receiver<TransactionMessage>,
    external_tx: Sender<TransactionMessage>,
}

impl NodeTransaction{
    pub fn new(external_tx:Sender<TransactionMessage>) -> Self{
        let (tx, rx) = channel();
        NodeTransaction{
            transaction_queue: BinaryHeap::new(),
            tx, rx,
            external_tx
        }
    }

    pub fn run(&mut self) {
        loop {
            match self.rx.recv_timeout(Duration::from_secs(1)) {
                Ok(message) => {
                    match message {
                        TransactionMessage::AddTransaction(transaction) => {
                            self.transaction_queue.push(transaction);
                        }
                        TransactionMessage::GetTransaction() => {
                            let chain = self.get_transactions();
                            if chain.len() > 0 {
                                self.external_tx.send(TransactionMessage::TransactionVec(chain)).unwrap();
                            }
                        }
                        (_) => ()
                    }
                },
                Err(err) => {
                    match err {
                        RecvTimeoutError=> {}
                        _ => {
                            error!("Unknown transaction message type: {}", err);
                        }
                    }
                }
            }
        }
    }

    pub fn get_transactions(&mut self) -> Vec<SerializedTransaction> {
        let mut transactions = Vec::new();
        for _ in 0..4 {
            if let Some(t) = self.transaction_queue.pop() {
                transactions.push(t);
            } else {
                break; // Нет элементов — выходим из цикла
            }
        }
        transactions
    }

    pub fn get_sender(&self) -> Sender<TransactionMessage> {
        return self.tx.clone();
    }
}