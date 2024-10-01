use std::sync::mpsc::Receiver;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    id: u64,
    sender: String,
    receiver: String,
    message: String,
    tax: f64,
}

impl Transaction {
    pub fn new(sender: String, receiver: String, message: String, tax: f64) -> Transaction {
        Transaction { id: 0, sender, receiver, message, tax }
    }
    pub fn to_string(&self) -> String {
        format!("{}", self.id)
    }
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    pub fn get_id(&self) -> u64 {
        self.id
    }

    pub fn get_receiver(&self) -> String {
        self.receiver.clone()
    }

    pub fn get_sender(&self) -> String {
        self.sender.clone()
    }

    pub fn get_tax(&self) -> f64 {
        self.tax
    }

}