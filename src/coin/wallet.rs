use std::panic::panic_any;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Clone)]
pub struct Wallet {
    open_key: String,
    close_key: String,
    amount: f64,
}

impl Wallet {
    pub fn new() -> Wallet {}
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    pub fn from_json(json_str: &str) -> Self {
        let result = serde_json::from_str(json_str);
        match result {
            Ok(result_map) => {
                return result_map;
            }
            Err(e) => {
                eprintln!("Error read wallet values from json");
                return Self::new();
            }
        }
    }

    pub fn get_open_key(&self) -> String {
        self.open_key.clone()
    }

    pub fn get_close_key(&self) -> String {
        self.close_key.clone()
    }

    pub fn get_amount(&self) -> f64 {
        self.amount
    }

    pub fn set_amount(&mut self, mount: f64) {
        if mount < 0f64 {
            panic!("value can't be negative");
        } else {
            self.amount = mount;
        }
    }
}