use rusqlite::{params, Connection, Result};
use crate::coin::node::blockchain::block::Block;
use crate::coin::node::blockchain::transaction::SerializedTransaction;

pub struct BlockDatabase {
    conn: Connection,
}

impl BlockDatabase {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.create_table()?;
        Ok(db)
    }

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

    /// Сохраняет блок в БД
    pub fn insert_block(&self, block: &Block) -> Result<()> {
        let tx_data = bincode::serialize(&block.get_transactions()).unwrap();

        self.conn.execute(
            "INSERT INTO blocks (id, time_create, transactions, previous_hash, nonce)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                block.get_id() as i64,
                block.get_datetime(),
                tx_data,
                block.get_previous_hash(),
                block.get_nonce()
            ],
        )?;
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
            let transactions: Vec<SerializedTransaction> = bincode::deserialize(&tx_blob).unwrap();

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
        let ids = stmt.query_map([], |row| row.get(0))?
            .collect::<Result<Vec<i64>, _>>()?;

        let mut blocks = Vec::new();
        for id in ids {
            blocks.push(self.get_block(id as usize)?);
        }
        Ok(blocks)
    }
}