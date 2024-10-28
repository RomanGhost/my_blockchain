use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use rand::rngs::OsRng;
use rsa::{RsaPrivateKey, RsaPublicKey};
use serde::{Deserialize, Serialize};
use rsa::pkcs1::{DecodeRsaPublicKey, EncodeRsaPublicKey};
use rsa::pkcs8::{DecodePrivateKey, EncodePrivateKey};
use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine as _};
use log::{info, warn, error};

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
        let private_key = match RsaPrivateKey::new(&mut rng, 2048) {
            Ok(private_key) => {
                info!("Private key successfully generated.");
                private_key
            }
            Err(e) => {
                error!("Unable to generate private key: {}", e);
                panic!("Unable to generate private key");
            }
        };

        let public_key = RsaPublicKey::from(&private_key);
        info!("Public key successfully generated from private key.");

        Wallet { public_key, private_key, amount: 0f64 }
    }

    // Сериализация кошелька
    pub fn serialize(&self) -> SerializedWallet {
        let public_key_base64 = self.get_public_key_string();
        let private_key_base64 = self.get_private_key_string();

        info!("Wallet successfully serialized.");

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
                info!("Wallet successfully deserialized from JSON.");

                let serialized_public_key = serialized_wallet.public_key.trim().to_string();
                let public_key = match RsaPublicKey::from_pkcs1_der(
                    &STANDARD_NO_PAD.decode(&serialized_public_key).unwrap()
                ) {
                    Ok(key) => key,
                    Err(e) => {
                        error!("Failed to decode public key: {}", e);
                        panic!("Error reading public key");
                    }
                };

                let serialized_private_key = serialized_wallet.private_key.trim().to_string();
                let private_key = match RsaPrivateKey::from_pkcs8_der(
                    &STANDARD_NO_PAD.decode(&serialized_private_key).unwrap()
                ) {
                    Ok(key) => key,
                    Err(e) => {
                        error!("Failed to decode private key: {}", e);
                        panic!("Error reading private key");
                    }
                };

                Wallet {
                    public_key,
                    private_key,
                    amount: serialized_wallet.amount,
                }
            }
            Err(e) => {
                error!("Error parsing JSON: {}. Creating new wallet.", e);
                Self::new()
            }
        }
    }

    // Возвращает публичный ключ в формате Base64
    pub fn get_public_key_string(&self) -> String {
        let public_key_der = self.public_key.to_pkcs1_der().unwrap();
        info!("Public key successfully converted to Base64.");
        STANDARD_NO_PAD.encode(public_key_der.as_bytes())
    }

    // Возвращает приватный ключ в формате Base64
    pub fn get_private_key_string(&self) -> String {
        let private_key_der = self.private_key.to_pkcs8_der().unwrap();
        info!("Private key successfully converted to Base64.");
        STANDARD_NO_PAD.encode(private_key_der.as_bytes())
    }

    // Получить публичный ключ
    pub fn get_public_key(&self) -> RsaPublicKey {
        self.public_key.clone()
    }

    // Получить приватный ключ
    pub fn get_private_key(&self) -> RsaPrivateKey {
        self.private_key.clone()
    }

    // Получить баланс кошелька
    pub fn get_amount(&self) -> f64 {
        self.amount
    }

    // Установить баланс кошелька
    pub fn set_amount(&mut self, amount: f64) {
        if amount < 0f64 {
            error!("Attempted to set a negative balance: {}", amount);
            panic!("Balance cannot be negative");
        } else {
            info!("Wallet balance updated to {}", amount);
            self.amount = amount;
        }
    }

    // Загрузка кошелька из файла
    pub fn load_from_file(file_path: &str) -> Wallet {
        if Path::new(file_path).exists() {
            info!("Loading wallet from file: {}", file_path);
            let mut file = match File::open(file_path) {
                Ok(file) => file,
                Err(e) => {
                    error!("Failed to open wallet file: {}", e);
                    panic!("Error opening file");
                }
            };

            let mut contents = String::new();
            match file.read_to_string(&mut contents) {
                Ok(_) => info!("Successfully read wallet from file."),
                Err(e) => {
                    error!("Error reading wallet file: {}", e);
                    panic!("Error reading file");
                }
            }

            Wallet::from_json(&contents)
        } else {
            // todo Создание папки если ее нет
            warn!("Wallet file not found, creating a new one at: {}", file_path);
            let wallet = Wallet::new();
            wallet.save_to_file(file_path);
            wallet
        }
    }

    // Сохранение кошелька в файл
    pub fn save_to_file(&self, file_path: &str) {
        let json_str = self.to_json();
        let mut file = match File::create(file_path) {
            Ok(file) => {
                info!("Creating new wallet file: {}", file_path);
                file
            }
            Err(e) => {
                error!("Failed to create wallet file: {}", e);
                panic!("Error creating file");
            }
        };

        match file.write_all(json_str.as_bytes()) {
            Ok(_) => info!("Wallet successfully saved to file."),
            Err(e) => {
                error!("Error writing wallet to file: {}", e);
                panic!("Error writing to file");
            }
        }
    }
}

// Структура для сериализованного кошелька
#[derive(Serialize, Deserialize)]
struct SerializedWallet {
    public_key: String,    // Публичный ключ в формате Base64
    private_key: String,   // Приватный ключ в формате Base64
    amount: f64,
}
