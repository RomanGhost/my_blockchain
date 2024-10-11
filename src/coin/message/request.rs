use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::coin::blockchain::block::Block;

// Новый тип сообщения для запроса последних N блоков
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LastNBlocksMessage {
    id: u64,
    n: usize, // Количество блоков, которые необходимо запросить
}
impl LastNBlocksMessage {
    pub fn new(n: usize) -> LastNBlocksMessage {
        LastNBlocksMessage { id: 0, n }
    }

    pub fn get_id(&self) -> u64 {
        self.id
    }

    pub fn set_id(&mut self, id: u64) {
        self.id = id;
    }

    pub fn get_n(&self) -> usize {
        self.n
    }
}

// Новый тип сообщения для запроса блоков до определенной даты
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlocksBeforeMessage {
    id: u64,
    time_stamp: DateTime<Utc>, // Запрашиваем все блоки до этого времени
}
impl BlocksBeforeMessage {
    pub fn new(time_stamp: DateTime<Utc>) -> BlocksBeforeMessage {
        BlocksBeforeMessage { id: 0, time_stamp }
    }

    pub fn get_id(&self) -> u64 {
        self.id
    }

    pub fn set_id(&mut self, id: u64) {
        self.id = id;
    }

    pub fn get_time(&self) -> DateTime<Utc> {
        self.time_stamp
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MessageFirstInfo {
    id: u64
}
impl MessageFirstInfo {
    pub fn new() -> MessageFirstInfo {
        MessageFirstInfo { id: 0, }
    }

    pub fn get_id(&self) -> u64 {
        self.id
    }

    pub fn set_id(&mut self, id: u64) {
        self.id = id;
    }
}

