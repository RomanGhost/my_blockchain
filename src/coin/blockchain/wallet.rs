use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use rand::rngs::OsRng;
use rsa::{RsaPrivateKey, RsaPublicKey};
use serde::{Deserialize, Serialize};
use rsa::pkcs1::{DecodeRsaPublicKey, EncodeRsaPublicKey};
use rsa::pkcs8::{DecodePrivateKey, EncodePrivateKey};
use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine as _};

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

    // Сериализация кошелька
    pub fn serialize(&self) -> SerializedWallet {
        let public_key_base64 = self.get_public_key_string();
        let private_key_base64 = self.get_private_key_string();

        SerializedWallet {
            public_key: public_key_base64,
            private_key: private_key_base64,
            amount: self.amount,
        }
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
                let serialized_public_key = serialized_wallet.public_key.trim().to_string();
                let public_key = RsaPublicKey::from_pkcs1_der(
                    &STANDARD_NO_PAD.decode(&serialized_public_key).unwrap()
                ).expect("Ошибка чтения публичного ключа");

                let serialized_private_key = serialized_wallet.private_key.trim().to_string();

                let private_key = RsaPrivateKey::from_pkcs8_der(
                    &STANDARD_NO_PAD.decode(&serialized_private_key).unwrap()
                ).expect("Ошибка чтения приватного ключа");

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

    // Возвращает публичный ключ в формате Base64
    pub fn get_public_key_string(&self) -> String {
        let public_key_der = self.public_key.to_pkcs1_der().unwrap();
        STANDARD_NO_PAD.encode(public_key_der.as_bytes())
    }

    // Возвращает приватный ключ в формате Base64
    pub fn get_private_key_string(&self) -> String {
        let private_key_der = self.private_key.to_pkcs8_der().unwrap();
        STANDARD_NO_PAD.encode(private_key_der.as_bytes())
    }

    pub fn get_public_key(&self) -> RsaPublicKey {
        self.public_key.clone()
    }

    pub fn get_private_key(&self) -> RsaPrivateKey {
        self.private_key.clone()
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

    // Загрузка кошелька из файла
    pub fn load_from_file(file_path: &str) -> Wallet {
        if Path::new(file_path).exists() {
            let mut file = File::open(file_path).expect("Не удалось открыть файл");
            let mut contents = String::new();
            file.read_to_string(&mut contents).expect("Ошибка чтения файла");
            Wallet::from_json(&contents)
        } else {
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
    public_key: String,    // Публичный ключ в формате Base64
    private_key: String,   // Приватный ключ в формате Base64
    amount: f64,
}