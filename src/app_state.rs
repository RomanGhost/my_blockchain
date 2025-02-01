use std::collections::BinaryHeap;
use std::sync::{Arc, Mutex, Condvar};
use std::sync::atomic::AtomicBool;
use crate::coin::blockchain::blockchain::Blockchain;
use crate::coin::blockchain::transaction::SerializedTransaction;
use crate::coin::blockchain::wallet::Wallet;
use crate::coin::server::protocol::peers::P2PProtocol;
use crate::coin::server::server::Server;


pub struct AppState {
    pub server: Server,
    pub p2p_protocol: Arc<Mutex<P2PProtocol>>,
    pub blockchain: Arc<Mutex<Blockchain>>,
    pub wallet: Wallet,
    pub queue: Arc<Mutex<BinaryHeap<SerializedTransaction>>>,
    pub running: Arc<AtomicBool>,
    pub mining_flag: Arc<(Mutex<bool>, Condvar)>, // Для управления майнингом
}

impl AppState {
    /// Останавливает майнинг, используя `mining_flag`
    pub fn stop_mining(&self) {
        let (lock, cvar) = &*self.mining_flag;
        let mut stop_flag = lock.lock().unwrap();
        *stop_flag = false;  // Остановка майнинга
        cvar.notify_all();    // Уведомление
    }

    /// Возобновляет майнинг, используя `mining_flag`
    pub fn start_mining(&self) {
        let (lock, cvar) = &*self.mining_flag;
        let mut stop_flag = lock.lock().unwrap();
        *stop_flag = true;    // Возобновление майнинга
        cvar.notify_all();    // Уведомление
    }
}
