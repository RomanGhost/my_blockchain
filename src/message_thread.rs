use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::sync::mpsc::Receiver;
use std::thread;
use std::thread::JoinHandle;
use crate::app_state::AppState;
use crate::coin::message::r#type::Message;

pub fn message_thread(app_state: Arc<AppState>, rx_server: Receiver<Message>) -> JoinHandle<()> {
    thread::spawn(move || {
        for received in rx_server {
            if !app_state.running.load(Ordering::SeqCst) {
                break; // Выход из цикла, если программа не работает.
            }

            match received {
                Message::RequestLastNBlocksMessage(message) => {
                    let n = message.get_n();
                    let blocks = app_state.blockchain.lock().unwrap().get_last_n_blocks(n);
                    app_state.p2p_protocol.lock().unwrap().response_chain(blocks);
                }
                Message::ResponseChainMessage(message) => {
                    let chain = message.get_chain();
                    for b in chain {
                        println!("{:?}", b);
                    }
                }

                Message::ResponseBlockMessage(message) => {
                    let is_force_block = message.is_force();
                    let new_block = message.get_block();
                    println!("Получен новый блок: {}", new_block.get_id());

                    // Останавливаем майнинг
                    app_state.stop_mining();

                    let mut chain = app_state.blockchain.lock().unwrap();

                    if is_force_block {
                        chain.add_force_block(new_block);
                    } else {
                        chain.add_block(new_block);
                    }

                    // Очищаем nonce после добавления блока
                    chain.clear_nonce();

                    // Перезапускаем майнинг
                    app_state.start_mining();
                }
                Message::ResponseTransactionMessage(message) => {
                    let new_transaction = message.get_transaction();
                    println!("Получена новая транзакция! > {:?}", new_transaction);
                    app_state.queue.lock().unwrap().push(new_transaction);
                    println!("Транзакция добавлена в очередь");
                }
                Message::ResponseTextMessage(message) => {
                    println!("Новое сообщение > {}", message.get_text());
                }
                _ => {
                    eprintln!("Неизвестный тип сообщения");
                }
            }
        }
    })
}
