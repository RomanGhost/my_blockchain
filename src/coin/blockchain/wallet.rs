use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use rand::rngs::OsRng;
use rsa::{RsaPrivateKey, RsaPublicKey};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use rsa::pkcs1::{DecodeRsaPublicKey, EncodeRsaPublicKey, LineEnding, DecodeRsaPrivateKey};
use rsa::pkcs8::EncodePrivateKey;

// Структура кошелька
#[derive(Clone)]
pub struct Wallet {
    public_key: RsaPublicKey,
    private_key: RsaPrivateKey,
    amount: f64,
}

impl Wallet {
    // Создание нового кошелька
    pub fn new() -> Wallet {
        let mut rng = OsRng;
        let private_key = RsaPrivateKey::new(&mut rng, 2048).expect("Ошибка генерации ключа");
        let public_key = RsaPublicKey::from(&private_key);
        Wallet { public_key, private_key, amount: 0f64 }
    }

    pub fn serialize(&self) -> SerializedWallet {
        let public_key_pem = self.get_public_key_pem();
        let private_key_pem = self.get_private_key_pem();

        let serialized_wallet = SerializedWallet {
            public_key: public_key_pem,
            private_key: private_key_pem,
            amount: self.amount,
        };
        serialized_wallet
    }

    // Преобразование кошелька в JSON
    pub fn to_json(&self) -> String {
        let serialized_wallet = self.serialize();
        serde_json::to_string(&serialized_wallet).unwrap()
    }

    // Создание кошелька из JSON
    pub fn from_json(json_str: &str) -> Self {
        let result: Result<SerializedWallet, serde_json::Error> = serde_json::from_str(json_str);
        match result {
            Ok(serialized_wallet) => {
                let public_key = RsaPublicKey::from_pkcs1_pem(&serialized_wallet.public_key)
                    .expect("Ошибка чтения публичного ключа");
                let private_key = RsaPrivateKey::from_pkcs1_pem(&serialized_wallet.private_key)
                    .expect("Ошибка чтения приватного ключа");
                Wallet {
                    public_key,
                    private_key,
                    amount: serialized_wallet.amount,
                }
            }
            Err(_) => {
                eprintln!("Ошибка при чтении значений из JSON. Создается новый кошелек.");
                Self::new()
            }
        }
    }

    pub fn get_public_key(&self) -> RsaPublicKey {
        self.public_key.clone()
    }
    pub fn get_public_key_pem(&self) -> String {
        let public_key_pem = self.public_key.to_pkcs1_pem(LineEnding::CRLF).unwrap();  // Сериализация публичного ключа в PEM
        public_key_pem
    }

    pub fn get_private_key(&self) -> RsaPrivateKey {
        self.private_key.clone()
    }
    pub fn get_private_key_pem(&self) -> String {
        match self.private_key.to_pkcs8_pem(LineEnding::CRLF) {
            Ok(private_key_pem) => private_key_pem.to_string(),
            Err(e) => {
                eprintln!("Ошибка сериализации приватного ключа: {}", e);
                String::new()
            }
        }
    }

    pub fn get_amount(&self) -> f64 {
        self.amount
    }

    pub fn set_amount(&mut self, amount: f64) {
        if amount < 0f64 {
            panic!("Значение не может быть отрицательным");
        } else {
            self.amount = amount;
        }
    }

    pub fn load_from_file(file_path: &str) -> Wallet {
        // Проверяем существует ли файл
        if Path::new(file_path).exists() {
            // Если файл существует, читаем его содержимое
            let mut file = File::open(file_path).expect("Не удалось открыть файл");
            let mut contents = String::new();
            file.read_to_string(&mut contents).expect("Ошибка чтения файла");

            // Пробуем десериализовать данные из JSON
            Wallet::from_json(&contents)
        } else {
            // Если файла нет, создаем новый кошелек и сохраняем его
            let wallet = Wallet::new();
            wallet.save_to_file(file_path);
            wallet
        }
    }

    // Сохранение кошелька в файл
    pub fn save_to_file(&self, file_path: &str) {
        let json_str = self.to_json();
        let mut file = File::create(file_path).expect("Не удалось создать файл");
        file.write_all(json_str.as_bytes()).expect("Ошибка записи в файл");
    }
}

// Структура для сериализованного кошелька
#[derive(Serialize, Deserialize)]
struct SerializedWallet {
    public_key: String,    // Публичный ключ в виде строки (PEM)
    private_key: String,   // Приватный ключ в виде строки (PEM)
    amount: f64,
}
