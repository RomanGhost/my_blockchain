use log::{debug, error}; // Добавлен импорт error для логирования ошибок
use rusqlite::{params, Connection, Result}; // Result здесь это rusqlite::Result
use crate::coin::node::blockchain::block::Block;
use crate::coin::node::blockchain::transaction::SerializedTransaction; // Убедитесь, что этот импорт есть, если он нужен для Block

// --- Структура BlockDatabase ---
pub struct BlockDatabase {
    conn: Connection,
}

impl BlockDatabase {

    /// Открывает или создает базу данных
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.create_table()?;
        Ok(db)
    }

    /// Создает таблицу блоков, если она не существует
    fn create_table(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS blocks (
                id INTEGER PRIMARY KEY,
                time_create INTEGER NOT NULL,
                transactions BLOB NOT NULL,
                previous_hash TEXT NOT NULL,
                nonce INTEGER NOT NULL
            )",
            [],
        )?;
        Ok(())
    }

    /// Сохраняет блок в БД (переписанная версия)
    /// Возвращает rusqlite::Result<()> для совместимости
    pub fn insert_block(&self, block: &Block) -> Result<()> {
        // 1. Сериализуем транзакции с обработкой ошибки
        let tx_data = match bincode::serialize(&block.get_transactions()) {
            Ok(data) => data, // Если сериализация успешна, используем данные
            Err(e) => {
                // Если произошла ошибка сериализации bincode:
                // Логируем ее для диагностики
                error!(
                    "Serialization failed for block ID {}: {}",
                    block.get_id(),
                    e
                );
                // Преобразуем ошибку bincode в ошибку rusqlite, чтобы соответствовать
                // типу возвращаемого значения функции. Используем ToSqlConversionFailure
                // как семантически близкий вариант (ошибка подготовки данных для SQL).
                // bincode::Error (Box<ErrorKind>) реализует типаж std::error::Error.
                return Err(rusqlite::Error::ToSqlConversionFailure(e));
            }
        };

        // 2. Выполняем вставку в базу данных
        // Используем более информативное сообщение для отладки
        debug!(
            "Inserting block ID {} into DB (PrevHash: {}, Nonce: {})",
            block.get_id(),
            block.get_previous_hash(),
            block.get_nonce()
        );

        // Выполняем SQL-запрос. Оператор '?' автоматически обработает
        // ошибки выполнения SQL (например, ошибка диска, нарушение ограничений UNIQUE)
        // и вернет rusqlite::Error в случае неудачи.
        let affected_rows = self.conn.execute(
            "INSERT INTO blocks (id, time_create, transactions, previous_hash, nonce)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                block.get_id() as i64,     // ID блока
                block.get_datetime(),      // Время создания
                tx_data,                   // Сериализованные транзакции (BLOB)
                block.get_previous_hash(), // Хеш предыдущего блока
                block.get_nonce()          // Nonce
            ],
        )?; // Если execute вернет Err, '?' прервет выполнение и вернет эту ошибку

        // (Опционально) Проверяем, что строка действительно была вставлена
        if affected_rows == 0 {
            error!("Block ID {} was not inserted into DB, execute returned 0 affected rows.", block.get_id());
            // Можно вернуть специфическую ошибку, если 0 вставленных строк считать ошибкой
            // return Err(rusqlite::Error::ExecuteReturnedTimeOut); // Пример, найти подходящую ошибку или создать свою
        } else {
            debug!("Block ID {} successfully inserted, affected rows: {}", block.get_id(), affected_rows);
        }


        // 3. Если оба шага (сериализация и выполнение SQL) прошли успешно, возвращаем Ok
        Ok(())
    }

    /// Загружает блок по ID
    pub fn get_block(&self, id: usize) -> Result<Block> {
        let mut stmt = self.conn.prepare(
            "SELECT id, time_create, transactions, previous_hash, nonce FROM blocks WHERE id = ?1"
        )?;
        let mut rows = stmt.query(params![id as i64])?;

        if let Some(row) = rows.next()? {
            let tx_blob: Vec<u8> = row.get(2)?;
            // Обрабатываем ошибку десериализации
            let transactions: Vec<SerializedTransaction> = match bincode::deserialize(&tx_blob) {
                Ok(txs) => txs,
                Err(e) => {
                    error!("Deserialization failed for block ID {}: {}", row.get::<_, i64>(0)?, e);
                    // Преобразуем в ошибку rusqlite
                    return Err(rusqlite::Error::FromSqlConversionFailure(
                        2, // Индекс колонки 'transactions'
                        rusqlite::types::Type::Blob,
                        e, // Ошибка bincode
                    ));
                }
            };

            Ok(Block::force_new(
                row.get::<_, i64>(0)? as usize,
                row.get(1)?,
                transactions,
                row.get(3)?,
                row.get(4)?,
            ))
        } else {
            Err(rusqlite::Error::QueryReturnedNoRows)
        }
    }

    /// Получить все блоки (по желанию)
    pub fn get_all_blocks(&self) -> Result<Vec<Block>> {
        let mut stmt = self.conn.prepare("SELECT id FROM blocks ORDER BY id")?;
        let ids = stmt.query_map([], |row| row.get::<_, i64>(0))?
            .collect::<std::result::Result<Vec<i64>, _>>()?; // Указываем полный путь к Result

        let mut blocks = Vec::with_capacity(ids.len()); // Оптимизация: выделяем память заранее
        for id_val in ids { // Используем другое имя переменной, чтобы не затенять id в get_block
            match self.get_block(id_val as usize) {
                Ok(block) => blocks.push(block),
                Err(e) => {
                    error!("Failed to retrieve or deserialize block with ID {}: {}", id_val, e);
                    // Решаем, что делать: пропустить блок или вернуть ошибку для всей операции
                    // Вариант 1: Пропустить и продолжить
                    continue;
                    // Вариант 2: Вернуть первую же ошибку (раскомментировать строку ниже)
                    // return Err(e);
                }
            }
        }
        Ok(blocks)
    }

}