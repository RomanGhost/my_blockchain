use std::sync::{Arc, Mutex, Condvar};
use std::sync::atomic::Ordering;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use crate::app_state::AppState;
use crate::coin::blockchain::transaction::SerializedTransaction;

pub fn mining_thread(app_state: Arc<AppState>) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut transactions: Vec<SerializedTransaction> = vec![]; // Буфер для транзакций

        loop {
            if !app_state.running.load(Ordering::SeqCst) {
                break; // Завершение работы, если приложение остановлено
            }

            // Заблокировать и получить флаг майнинга
            let (lock, cvar) = &*app_state.mining_flag;
            let mut is_mining = lock.lock().unwrap();

            // Если майнинг отключен, подождем пока флаг не будет установлен в true
            if !*is_mining {
                is_mining = cvar.wait(is_mining).unwrap();
            }

            // Берем блокчейн и очередь транзакций
            let mut chain = app_state.blockchain.lock().unwrap();
            let mut lock_queue = app_state.queue.lock().unwrap();

            // Заполняем пул транзакций (до 4 штук)
            while transactions.len() < 4 && !lock_queue.is_empty() {
                if let Some(transaction) = lock_queue.pop() {
                    transactions.push(transaction);
                } else {
                    println!("Нет доступных транзакций для обработки.");
                    break;
                }
            }

            // Очищаем nonce, если есть транзакции для майнинга
            if !transactions.is_empty() {
                chain.clear_nonce();
            }

            drop(lock_queue); // Снимаем блокировку с очереди транзакций

            // Проводим вычисления майнинга (Proof of Work)
            let iteration_result = chain.proof_of_work(transactions.clone());

            // Если найден новый блок, рассылаем его другим узлам
            if iteration_result {
                if let Ok(last_block) = chain.get_last_block() {
                    app_state.p2p_protocol.lock().unwrap().response_block(last_block, false);
                    transactions.clear(); // Очищаем пул транзакций после успешного майнинга
                    println!("Отправлен новый блок");
                }
            }

            // Короткая пауза перед следующей итерацией майнинга
            thread::sleep(Duration::from_millis(1));
        }
    })
}
