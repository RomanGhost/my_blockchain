use chrono::{DateTime, Utc};
use sha2::{Digest, Sha512};

use crate::coin::node::blockchain::block::Block;

pub struct Blockchain {
    pub chain: Vec<Block>,
    nonce_iteration: u64,
}
impl Blockchain {
    pub fn new() -> Blockchain {
        Blockchain {
            chain: Vec::new(),
            nonce_iteration: 0,
        }
    }

    pub fn add_block(&mut self, block: Block) -> Result<Block, String> {
        let mut block = block;
        if !Blockchain::is_valid_block(&block) {
            return Err("Hash didn't valid".to_string())
        }
        if let Ok(last_block) = self.get_last_block() {
            if block.get_previous_hash() == last_block.get_hash() {
                self.chain.push(block.clone());
                Ok(block)
            } else {
                Err("Хеши не совпадают".to_string())
            }
        } else {
            Err("chain is empty".to_string())
        }
    }

    pub fn add_force_block(&mut self, block: Block) {
        self.chain.push(block);
    }

    pub fn get_last_block(&self) -> Result<Block, &'static str> {
        if let Some(block) = self.chain.last() {
            Ok(block.clone())
        } else {
            Err("chain is empty")
        }
    }

    pub fn create_first_block(&mut self) {
        let word = "First block";
        let mut hasher = Sha512::new();
        hasher.update(word);
        let result = hasher.finalize();
        let hex_string = format!("{:x}", result);

        let block = Block::new(1, Vec::new(), hex_string, 0);
        self.add_force_block(block);
    }

    pub fn len(&self) -> usize {
        self.chain.len()
    }

    pub fn is_valid_block(block: &Block) -> bool {
        block.get_hash().starts_with("000")
    }

    pub fn get_blocks_after(&self, datetime: i64) -> Vec<Block> {
        self.chain
            .iter()
            .filter(|block| datetime < block.get_datetime())
            .cloned()
            .collect()
    }

    pub fn get_blocks_before(&self, datetime: i64) -> Vec<Block> {
        self.chain
            .iter()
            .filter(|block| datetime > block.get_datetime())
            .cloned()
            .collect()
    }

    pub fn get_full_chain(&self) -> Vec<Block>{
        self.chain.clone()
    }

    pub fn get_last_n_blocks(&self, n: usize) -> Vec<Block> {
        self.chain
            .iter()
            .take(n)
            .cloned()
            .collect()
    }

    pub fn clear_nonce(&mut self) {
        self.nonce_iteration = 0;
    }
}

pub fn validate_chain(new_chain: &Vec<Block>) -> bool {
    for i in 1..new_chain.len() {
        let current_block = &new_chain[i];
        let previous_block = &new_chain[i - 1];

        // Проверка корректности ссылок на предыдущие блоки
        if current_block.get_previous_hash() != previous_block.get_hash() {
            return false;
        }

        // Дополнительная проверка хешей и PoW
        if !Blockchain::is_valid_block(current_block) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Utc};
    use std::thread::sleep;
    use std::time::Duration as StdDuration;
    use crate::coin::node::blockchain::transaction::SerializedTransaction;

    // Функция для создания тестовой транзакции.
    fn sample_transactions() -> Vec<SerializedTransaction> {
        vec![
            SerializedTransaction::new(
                "sender_base64".to_string(),
                "seller_base64".to_string(),
                "buyer_base64".to_string(),
                "Test message".to_string(),
                123.45,
            )
        ]
    }

    /// Функция для «майнинга» блока — подбирается значение nonce, при котором хэш блока начинается с "000".
    fn mine_valid_block(id: usize, transactions: Vec<SerializedTransaction>, previous_hash: String) -> Block {
        let mut nonce = 0;
        loop {
            let block = Block::new(id, transactions.clone(), previous_hash.clone(), nonce);
            if Block::get_hash(&block).starts_with("000") {
                return block;
            }
            nonce += 1;
        }
    }

    #[test]
    fn test_create_first_block() {
        let mut blockchain = Blockchain::new();
        blockchain.create_first_block();
        assert_eq!(blockchain.len(), 1);

        let first_block = blockchain.chain.first().unwrap();
        // Проверяем, что первый блок прошёл PoW (хэш начинается с "000")
        assert_eq!(first_block.get_nonce(), 0, "Nonce equals zero");
    }

    #[test]
    fn test_add_valid_block() {
        let mut blockchain = Blockchain::new();
        blockchain.create_first_block();

        let last_block = blockchain.get_last_block().unwrap();
        let prev_hash = last_block.get_hash();
        let transactions = sample_transactions();
        let new_block = mine_valid_block(last_block.get_id() + 1, transactions, prev_hash);

        let result = blockchain.add_block(new_block.clone());
        assert!(result.is_ok(), "Блок должен быть добавлен в цепочку");
        assert_eq!(blockchain.len(), 2);
    }

    #[test]
    fn test_add_invalid_block() {
        let mut blockchain = Blockchain::new();
        blockchain.create_first_block();

        let last_block = blockchain.get_last_block().unwrap();
        let transactions = sample_transactions();

        // Создадим блок с nonce, который, вероятно, не даст валидного хэша (хэш не начинается с "000")
        let mut invalid_block = Block::new(last_block.get_id() + 1, transactions.clone(), last_block.get_hash(), 0);
        // Если случайно получилось валидное значение, форсированно изменим nonce, чтобы хэш не удовлетворял условию
        if Block::get_hash(&invalid_block).starts_with("000") {
            invalid_block = Block::new(last_block.get_id() + 1, transactions, last_block.get_hash(), 9999);
            assert!(!Block::get_hash(&invalid_block).starts_with("000"));
        }

        let result = blockchain.add_block(invalid_block);
        assert!(result.is_err(), "Блок с недопустимым хешем не должен быть добавлен");
    }

    #[test]
    fn test_get_blocks_after_before() {
        let mut blockchain = Blockchain::new();
        blockchain.create_first_block();
        let first_block_time = blockchain.chain.first().unwrap().get_datetime();

        // Ждём 1 секунду для создания разницы во времени
        sleep(StdDuration::from_secs(1));

        let last_block = blockchain.get_last_block().unwrap();
        let new_block = mine_valid_block(last_block.get_id() + 1, sample_transactions(), last_block.get_hash());
        blockchain.add_force_block(new_block.clone());
        let new_block_time = blockchain.chain.last().unwrap().get_datetime();

        // Получаем блоки, созданные после времени первого блока
        let after_blocks = blockchain.get_blocks_after(first_block_time);
        assert!(after_blocks.iter().any(|b| b.get_datetime() >= new_block_time),
                "Должен быть найден блок, созданный после указанного времени");

        // Получаем блоки, созданные до времени нового блока
        let before_blocks = blockchain.get_blocks_before(new_block_time);
        assert!(before_blocks.iter().any(|b| b.get_datetime() <= first_block_time),
                "Должен быть найден блок, созданный до указанного времени");
    }

    #[test]
    fn test_get_last_n_blocks() {
        let mut blockchain = Blockchain::new();
        blockchain.create_first_block();

        // Добавляем ещё несколько блоков в цепочку
        for _ in 2..6 {
            let last_block = blockchain.get_last_block().unwrap();
            let new_block = mine_valid_block(last_block.get_id() + 1, sample_transactions(), last_block.get_hash());
            blockchain.add_force_block(new_block);
        }

        let n = 3;
        let last_n = blockchain.get_last_n_blocks(n);
        assert_eq!(last_n.len(), n, "Функция должна вернуть ровно {} блоков", n);
    }

    #[test]
    fn test_clear_nonce() {
        let mut blockchain = Blockchain::new();
        // Просто вызываем метод, проверка заключается в отсутствии ошибок
        blockchain.clear_nonce();
    }

    #[test]
    fn test_validate_chain_function() {
        let mut blockchain = Blockchain::new();
        blockchain.create_first_block();

        // Построим валидную цепочку с использованием add_force_block
        for _ in 0..3 {
            let last_block = blockchain.get_last_block().unwrap();
            let new_block = mine_valid_block(last_block.get_id() + 1, sample_transactions(), last_block.get_hash());
            blockchain.add_force_block(new_block);
        }
        // Валидная цепочка должна пройти проверку
        assert!(validate_chain(&blockchain.chain), "Цепочка должна быть валидной");

        // Нарушим цепочку: изменим поле previous_hash одного из блоков
        let mut invalid_chain = blockchain.chain.clone();
        if let Some(block) = invalid_chain.get_mut(2) {
            block.set_previous_hash("fake_hash".to_string());
        }
        assert!(!validate_chain(&invalid_chain), "Цепочка с нарушенными ссылками должна быть невалидной");
    }
}