use std::cmp::Ordering;
use serde::{Serialize, Deserialize};
use rsa::pkcs1::{DecodeRsaPublicKey, EncodeRsaPublicKey, LineEnding};
use base64;
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
    signature: String,
}

impl Transaction {
    // Создание новой транзакции с конвертацией ключей в строковый формат
    pub fn new(sender: RsaPublicKey, receiver: RsaPublicKey, message: String, tax: f64) -> Transaction {
        Transaction {
            id: 0,
            sender,
            receiver,
            message,
            tax,
            signature: "".to_string(),
        }
    }

    // Подпись транзакции с использованием приватного ключа
    pub fn sign(&mut self, private_key: RsaPrivateKey) {
        // Сериализация данных для подписи
        let message = format!("{}:{}:{}",
                              self.sender.to_pkcs1_pem(LineEnding::LF).unwrap(),
                              self.receiver.to_pkcs1_pem(LineEnding::LF).unwrap(),
                              self.message,
        );
        let message_bytes = message.into_bytes();

        // Хеширование сообщения
        let mut hasher = Sha256::new();
        hasher.update(message_bytes);
        let hashed_message = hasher.finalize();

        // Подпись хеша
        let padding = PaddingScheme::new_pkcs1v15_sign_raw();
        let signature_bytes = private_key.sign(padding, &hashed_message).expect("Не удалось подписать сообщение");

        // Кодирование подписи в Base64 для сохранения в строку
        self.signature = base64::encode(signature_bytes); // Преобразуем Vec<u8> в Base64 строку
    }

    // Проверка подписи
    pub fn verify(&self) -> bool {
        // Сериализация данных для проверки подписи
        let message = format!("{}:{}:{}",
                              self.sender.to_pkcs1_pem(LineEnding::LF).unwrap(),
                              self.receiver.to_pkcs1_pem(LineEnding::LF).unwrap(),
                              self.message,
        );
        let message_bytes = message.into_bytes();

        // Хеширование сообщения
        let mut hasher = Sha256::new();
        hasher.update(message_bytes);
        let hashed_message = hasher.finalize();

        // Декодирование подписи из Base64 обратно в бинарный формат
        let signature_bytes = base64::decode(&self.signature).expect("Ошибка декодирования подписи из Base64");

        // Проверка подписи
        let public_key = self.sender.clone();
        let padding = PaddingScheme::new_pkcs1v15_sign_raw();
        public_key.verify(padding, &hashed_message, &signature_bytes).is_ok()
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

    pub fn deserialize(serialized_transaction: SerializedTransaction) -> Result<Self, String> {
        // Десериализация публичного ключа отправителя
        let sender = RsaPublicKey::from_pkcs1_pem(&serialized_transaction.sender)
            .map_err(|_| "Ошибка чтения публичного ключа отправителя".to_string())?;

        // Десериализация публичного ключа получателя
        let receiver = RsaPublicKey::from_pkcs1_pem(&serialized_transaction.receiver)
            .map_err(|_| "Ошибка чтения публичного ключа получателя".to_string())?;

        Ok(Transaction {
            id: serialized_transaction.id,
            sender,
            receiver,
            message: serialized_transaction.message,
            tax: serialized_transaction.tax,
            signature: serialized_transaction.signature,
        })
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
                Transaction::deserialize(serialized_transaction)
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
    pub signature: String,
}

impl Eq for SerializedTransaction {}

impl PartialEq for SerializedTransaction {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id &&
            self.sender == other.sender &&
            self.receiver == other.receiver &&
            self.message == other.message &&
            self.tax == other.tax &&
            self.signature == other.signature
    }
}

// Реализуем Ord для сортировки по приоритету
impl Ord for SerializedTransaction {
    fn cmp(&self, other: &Self) -> Ordering {
        self.tax.partial_cmp(&other.tax).unwrap_or(Ordering::Equal) // Сортировка по возрастанию
    }
}

impl PartialOrd for SerializedTransaction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
