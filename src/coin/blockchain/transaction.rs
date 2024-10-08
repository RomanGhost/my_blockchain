use serde::{Serialize, Deserialize};
use rsa::pkcs1::{DecodeRsaPublicKey, EncodeRsaPublicKey, LineEnding};

use rsa::{RsaPrivateKey, RsaPublicKey, PaddingScheme, PublicKey};
use sha2::Sha256;
use rsa::signature::digest::Digest;

#[derive(Debug, Clone)]
pub struct Transaction {
    id: u64,
    sender: RsaPublicKey,
    receiver: RsaPublicKey,
    message: String,
    tax: f64,
    signature: Vec<u8>,
}

impl Transaction {
    // Создание новой транзакции с конвертацией ключей в строковый формат
    pub fn new(sender: RsaPublicKey, receiver: RsaPublicKey, message: String, tax: f64, signature: Vec<u8>) -> Transaction {
        Transaction {
            id: 0,
            sender,
            receiver,
            message,
            tax,
            signature,
        }
    }

    pub fn sign(&mut self, private_key: RsaPrivateKey) {
        // Исходное сообщение
        let message = self.to_string();
        let message = message.into_bytes();

        // Хеширование сообщения
        let mut hasher = Sha256::new();
        hasher.update(message);
        let hashed_message = hasher.finalize();

        // Подпись хеша
        let padding = PaddingScheme::new_pkcs1v15_sign_raw();
        self.signature = private_key.sign(padding, &hashed_message).expect("Не удалось подписать сообщение");
    }

    pub fn verify(&self) {
        // Исходное сообщение
        let message = self.to_string();
        let message = message.into_bytes();

        // Хеширование сообщения
        let mut hasher = Sha256::new();
        hasher.update(message);
        let hashed_message = hasher.finalize();

        // Проверка подписи
        let signature = self.signature.clone();
        let public_key = self.sender.clone();
        let padding = PaddingScheme::new_pkcs1v15_sign_raw();
        let is_valid = public_key.verify(padding, &hashed_message, &signature).is_ok();

        // Результат проверки
        if is_valid {
            println!("Подпись верна!");
        } else {
            println!("Подпись неверна!");
        }
    }


    // Преобразование транзакции в строку (для демонстрации)
    pub fn to_string(&self) -> String {
        let sender_pem = self.sender.to_pkcs1_pem(LineEnding::LF).unwrap();
        let receiver_pem = self.receiver.to_pkcs1_pem(LineEnding::LF).unwrap();

        format!("{}:{}:{}:{}", sender_pem, receiver_pem, self.message, self.tax)
    }

    pub fn serialize(&self) -> SerializedTransaction {
        let sender_pem = self.sender.to_pkcs1_pem(LineEnding::LF).unwrap();
        let receiver_pem = self.receiver.to_pkcs1_pem(LineEnding::LF).unwrap();

        SerializedTransaction {
            id: self.id,
            sender: sender_pem,
            receiver: receiver_pem,
            message: self.message.clone(),
            tax: self.tax,
            signature: self.signature.clone(),
        }
    }

    // Преобразование транзакции в JSON
    pub fn to_json(&self) -> String {
        let serialized_transaction = self.serialize();
        serde_json::to_string(&serialized_transaction).unwrap()
    }

    pub fn from_json(json_str: &str) -> Result<Self, String> {
        // Парсинг JSON в структуру SerializedTransaction
        let result: Result<SerializedTransaction, serde_json::Error> = serde_json::from_str(json_str);

        match result {
            Ok(serialized_transaction) => {
                // Десериализация публичного ключа отправителя
                let sender = RsaPublicKey::from_pkcs1_pem(&serialized_transaction.sender)
                    .map_err(|_| "Ошибка чтения публичного ключа отправителя".to_string())?;

                // Десериализация публичного ключа получателя
                let receiver = RsaPublicKey::from_pkcs1_pem(&serialized_transaction.receiver)
                    .map_err(|_| "Ошибка чтения публичного ключа получателя".to_string())?;

                // Создаем объект Transaction
                Ok(Transaction {
                    id: serialized_transaction.id,
                    sender,
                    receiver,
                    message: serialized_transaction.message,
                    tax: serialized_transaction.tax,
                    signature: serialized_transaction.signature,
                })
            }
            Err(err) => {
                // Возвращаем строку ошибки при неудачной десериализации JSON
                Err(format!("Ошибка при чтении JSON: {}", err))
            }
        }
    }

    // Получение ID транзакции
    pub fn get_id(&self) -> u64 {
        self.id
    }

    // Получение публичного ключа получателя (конвертация обратно из PEM)
    pub fn get_receiver(&self) -> RsaPublicKey {
        self.receiver.clone()
    }

    // Получение публичного ключа отправителя (конвертация обратно из PEM)
    pub fn get_sender(&self) -> RsaPublicKey {
        self.sender.clone()
    }

    // Получение комиссии транзакции
    pub fn get_tax(&self) -> f64 {
        self.tax
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SerializedTransaction {
    pub id: u64,
    pub sender: String,
    pub receiver: String,
    pub message: String,
    pub tax: f64,
    pub signature: Vec<u8>,
}
