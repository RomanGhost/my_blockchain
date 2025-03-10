use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::sync::mpsc::Receiver;
use std::thread;
use std::thread::JoinHandle;
use log::{info, warn};
use crate::app_state::AppState;
use crate::coin::blockchain::blockchain::validate_chain;
use crate::coin::blockchain::transaction::Transaction;
use crate::coin::server::protocol::message::r#type::Message;

pub fn message_thread(app_state: Arc<AppState>, rx_server: Receiver<Message>) -> JoinHandle<()> {
    thread::spawn(move || {
        for received in rx_server {
            if !app_state.running.load(Ordering::SeqCst) {
                break; // Выход из цикла, если программа не закончила работу
            }

            match received {
                Message::RequestLastNBlocksMessage(message) => {
                    info!("Get block chain");
                    let n = message.get_n();
                    app_state.stop_mining();
                    let blocks = app_state.blockchain.lock().unwrap().get_last_n_blocks(n);
                    app_state.p2p_protocol.lock().unwrap().response_chain(blocks);
                    app_state.start_mining();
                }

                Message::RequestBlocksBeforeMessage(message) => {
                    info!("Get messages before: {}", message.get_time());
                    app_state.stop_mining();
                    let blocks = app_state.blockchain.lock().unwrap().get_blocks_before(message.get_time());
                    app_state.p2p_protocol.lock().unwrap().response_chain(blocks);
                    app_state.start_mining();
                }

                // Обработка ответа с цепочкой блоков
                Message::ResponseChainMessage(message) => {
                    let new_chain = message.get_chain();
                    println!("Получена цепочка с {} блоками", new_chain.len());

                    // Остановка майнинга во время обработки цепочки
                    app_state.stop_mining();

                    // Захват локальной цепочки блоков
                    let mut chain = app_state.blockchain.lock().unwrap();

                    let new_chain_last_id = new_chain.last().map_or(0, |block| block.get_id());
                    let local_chain_last_id = chain.get_last_block().map_or(0, |block| block.get_id());

                    if new_chain_last_id > local_chain_last_id {
                        warn!("Новая цепочка длиннее локальной.");
                    } else {
                        info!("Локальная цепочка длиннее или равна новой.");
                    }

                    // Проверка на совпадение длин и выбор лучшей цепочки
                    if new_chain_last_id > local_chain_last_id {
                        let n = new_chain.len();
                        let local_chain = chain.get_last_n_blocks(n);
                        if validate_chain(&local_chain, &new_chain) {
                            info!("Цепочка валидна, обновление...");
                            chain.chain = new_chain;
                        } else {
                            warn!("Полученная цепочка невалидна");
                        }
                    } else {
                        info!("Полученная цепочка короче или равна текущей, обновление не требуется");
                    }

                    // Перезапуск майнинга после синхронизации
                    app_state.start_mining();
                }

                Message::ResponseBlockMessage(message) => {
                    let is_force_block = message.is_force();
                    let new_block = message.get_block();
                    println!("Получен новый блок: {}", new_block.get_id());

                    // Остановка майнинга
                    app_state.stop_mining();

                    let mut chain = app_state.blockchain.lock().unwrap();

                    // Получаем список транзакций из нового блока
                    let block_transactions = new_block.get_transactions();

                    let mut transaction_queue = app_state.queue.lock().unwrap();
                    // Удаляем транзакции из очереди, которые есть в новом блоке
                    transaction_queue.retain(|tx| !block_transactions.contains(tx));
                    //
                    // println!("Удалено {} транзакций из очереди", block_transactions.len());

                    if is_force_block {
                        chain.add_force_block(new_block);
                    } else {
                        if let Err(e) = chain.add_block(new_block) {
                            println!("{}", e);
                            app_state.p2p_protocol.lock().unwrap().request_chain(10);
                        }
                    }

                    // Очищаем nonce после добавления блока
                    chain.clear_nonce();

                    // Перезапуск майнинга
                    app_state.start_mining();
                }
                Message::ResponseTransactionMessage(message) => {
                    let new_transaction = message.get_transaction();
                    println!("Получена новая транзакция! > {:?}", new_transaction);
                    let transaction = Transaction::deserialize(new_transaction).unwrap();
                    let is_valid = transaction.verify();
                    if is_valid {
                        let serialize = transaction.serialize();
                        app_state.queue.lock().unwrap().push(serialize);
                        info!("Транзакция добавлена в очередь");
                    } else {
                        warn!("Транзакция не валидна");
                    }
                }
                Message::ResponsePeerMessage(message) => {
                    for addr in message.get_peers() {
                        let port = "7878";
                        let _ = app_state.server.connect(addr.as_str(), port);;
                    }
                }
                Message::ResponseTextMessage(message) => {
                    println!("Новое сообщение > {}", message.get_text());
                }
                _ => {
                    warn!("Неизвестный тип сообщения");
                }
            }
        }
    })
}
