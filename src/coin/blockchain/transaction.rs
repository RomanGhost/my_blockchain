use std::cmp::Ordering;
use std::fmt;
use std::fmt::Formatter;
use serde::{Serialize, Deserialize};
use rsa::pkcs1::{DecodeRsaPublicKey, EncodeRsaPublicKey, LineEnding};
use base64;
use base64::Engine;
use base64::engine::general_purpose::STANDARD_NO_PAD;
use rsa::{RsaPrivateKey, RsaPublicKey, PaddingScheme, PublicKey};
use sha2::Sha256;
use rsa::signature::digest::Digest;

#[derive(Debug, Clone)]
pub struct Transaction {
    id: u64,
    sender: RsaPublicKey,
    receiver: RsaPublicKey,
    message: String,
    transfer: f64,
    tax: f64,
    signature: String,
}

impl Transaction {
    // Создание новой транзакции с конвертацией ключей в строковый формат
    pub fn new(sender_base64: String, receiver_base64: String, message: String, transfer: f64, tax: f64) -> Transaction {
        let sender = RsaPublicKey::from_pkcs1_der(
            &STANDARD_NO_PAD.decode(&sender_base64).unwrap()
        ).expect("Ошибка чтения ключа отправителя");

        let receiver = RsaPublicKey::from_pkcs1_der(
            &STANDARD_NO_PAD.decode(&receiver_base64).unwrap()
        ).expect("Ошибка чтения ключа получателя");

        Transaction {
            id: 0,
            sender,
            receiver,
            message,
            transfer,
            tax,
            signature: "".to_string(),
        }
    }

    // Подпись транзакции с использованием приватного ключа
    pub fn sign(&mut self, private_key: RsaPrivateKey) {
        let message = self.to_string();
        let message_bytes = message.into_bytes();

        let mut hasher = Sha256::new();
        hasher.update(message_bytes);
        let hashed_message = hasher.finalize();

        let padding = PaddingScheme::new_pkcs1v15_sign_raw();
        let signature_bytes = private_key.sign(padding, &hashed_message).expect("Не удалось подписать сообщение");

        self.signature = STANDARD_NO_PAD.encode(signature_bytes);
    }

    // Проверка подписи
    pub fn verify(&self) -> bool {
        let message = self.to_json();
        let message_bytes = message.into_bytes();

        let mut hasher = Sha256::new();
        hasher.update(message_bytes);
        let hashed_message = hasher.finalize();

        let signature_bytes = STANDARD_NO_PAD.decode(&self.signature).expect("Ошибка декодирования подписи из Base64");

        let public_key = self.sender.clone();
        let padding = PaddingScheme::new_pkcs1v15_sign_raw();
        public_key.verify(padding, &hashed_message, &signature_bytes).is_ok()
    }

    pub fn serialize(&self) -> SerializedTransaction {
        let sender_der = self.sender.to_pkcs1_der().unwrap();
        let sender_base64 = STANDARD_NO_PAD.encode(sender_der.as_bytes());

        let receiver_der = self.receiver.to_pkcs1_der().unwrap();
        let receiver_base64 = STANDARD_NO_PAD.encode(receiver_der.as_bytes());

        SerializedTransaction {
            id: self.id,
            sender: sender_base64,
            receiver: receiver_base64,
            message: self.message.clone(),
            transfer: self.transfer,
            tax: self.tax,
            signature: self.signature.clone(),
        }
    }

    pub fn deserialize(serialized_transaction: SerializedTransaction) -> Result<Self, String> {
        let sender_base64 = serialized_transaction.sender;
        let sender = RsaPublicKey::from_pkcs1_der(
            &STANDARD_NO_PAD.decode(&sender_base64).unwrap()
        ).expect("Ошибка чтения ключа отправителя");

        let receiver_base64 = serialized_transaction.receiver;
        let receiver = RsaPublicKey::from_pkcs1_der(
            &STANDARD_NO_PAD.decode(&receiver_base64).unwrap()
        ).expect("Ошибка чтения ключа получателя");

        Ok(Transaction {
            id: serialized_transaction.id,
            sender,
            receiver,
            message: serialized_transaction.message,
            transfer: serialized_transaction.transfer,
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
        let result: Result<SerializedTransaction, serde_json::Error> = serde_json::from_str(json_str);

        match result {
            Ok(serialized_transaction) => {
                Transaction::deserialize(serialized_transaction)
            }
            Err(err) => {
                Err(format!("Ошибка при чтении JSON: {}", err))
            }
        }
    }

    // Получение ID транзакции
    pub fn get_id(&self) -> u64 {
        self.id
    }

    // Получение публичного ключа получателя
    pub fn get_receiver(&self) -> RsaPublicKey {
        self.receiver.clone()
    }

    // Получение публичного ключа отправителя
    pub fn get_sender(&self) -> RsaPublicKey {
        self.sender.clone()
    }

    // Получение комиссии транзакции
    pub fn get_tax(&self) -> f64 {
        self.tax
    }

    // Получение суммы перевода
    pub fn get_transfer(&self) -> f64 {
        self.transfer
    }

    // Получение сообщения транзакции
    pub fn get_message(&self) -> String {
        self.message.clone()
    }
}

impl fmt::Display for Transaction{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let sender_pem = self.sender.to_pkcs1_pem(LineEnding::LF).unwrap();
        let receiver_pem = self.receiver.to_pkcs1_pem(LineEnding::LF).unwrap();

        write!(f, "{}:{}:{}:{}", sender_pem, receiver_pem, self.message, self.tax)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SerializedTransaction {
    pub id: u64,
    pub sender: String,
    pub receiver: String,
    pub message: String,
    pub transfer: f64,
    pub tax: f64,
    pub signature: String,
}

impl SerializedTransaction {
    pub fn new(sender_base64: String, receiver_base64: String, message: String, transfer: f64, tax: f64) -> SerializedTransaction {
        let sender_base64 = sender_base64.trim().to_string();
        let receiver_base64 = receiver_base64.trim().to_string();

        SerializedTransaction {
            id: 0,
            sender: sender_base64,
            receiver: receiver_base64,
            message,
            transfer,
            tax,
            signature: "".to_string(),
        }
    }

    pub fn get_receiver(&self) -> String {
        self.receiver.clone()
    }

    pub fn get_sender(&self) -> String {
        self.sender.clone()
    }

    pub fn get_tax(&self) -> f64 {
        self.tax
    }

    pub fn get_transfer(&self) -> f64 {
        self.transfer
    }
}

impl Eq for SerializedTransaction {}

impl PartialEq for SerializedTransaction {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id &&
            self.sender == other.sender &&
            self.receiver == other.receiver &&
            self.message == other.message &&
            self.tax == other.tax &&
            self.transfer == other.transfer &&
            self.signature == other.signature
    }
}

// Реализуем Ord для сортировки по приоритету
impl Ord for SerializedTransaction {
    fn cmp(&self, other: &Self) -> Ordering {
        self.tax.partial_cmp(&other.tax).unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for SerializedTransaction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
