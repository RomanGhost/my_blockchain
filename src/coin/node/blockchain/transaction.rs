use std::cmp::Ordering;
use std::fmt;
use std::fmt::Formatter;

use base64;
use base64::Engine;
use base64::engine::general_purpose::STANDARD_NO_PAD;
use rsa::{PaddingScheme, PublicKey, RsaPrivateKey, RsaPublicKey};
use rsa::pkcs1::{DecodeRsaPublicKey, EncodeRsaPublicKey, LineEnding};
use rsa::signature::digest::Digest;
use serde::{Deserialize, Serialize};
use sha2::Sha256;

#[derive(Debug, Clone)]
pub struct Transaction {
    sender: RsaPublicKey,
    buyer: RsaPublicKey,
    seller: RsaPublicKey,
    message: String,
    transfer: f64,
    signature: String,
}

impl Transaction {
    // Создание новой транзакции с конвертацией ключей в строковый формат
    pub fn new(sender_base64: String, seller_base64: String, buyer_base64: String, message: String, transfer: f64) -> Transaction {
        let sender = RsaPublicKey::from_pkcs1_der(
            &STANDARD_NO_PAD.decode(&sender_base64).unwrap()
        ).expect("Ошибка чтения ключа отправителя");

        let buyer = RsaPublicKey::from_pkcs1_der(
            &STANDARD_NO_PAD.decode(&buyer_base64).unwrap()
        ).expect("Ошибка чтения ключа покупателя");

        let seller = RsaPublicKey::from_pkcs1_der(
            &STANDARD_NO_PAD.decode(&seller_base64).unwrap()
        ).expect("Ошибка чтения ключа продавца");

        Transaction {
            sender,
            buyer,
            seller,
            message,
            transfer,
            signature: "".to_string(),
        }
    }

    // Подпись транзакции с использованием приватного ключа
    pub fn sign(&mut self, private_key: RsaPrivateKey) {
        let sender_der = self.sender.to_pkcs1_der().unwrap();
        let sender_base64 = STANDARD_NO_PAD.encode(sender_der.as_bytes());
        // Собираем только данные, которые участвуют в подписи
        let data_to_sign = format!("{}:{}:{}", sender_base64, self.message, self.transfer);
        let message_bytes = data_to_sign.into_bytes();

        // Хешируем данные
        let mut hasher = Sha256::new();
        hasher.update(message_bytes);
        let hashed_message = hasher.finalize();

        // Создаем подпись
        let padding = PaddingScheme::new_pkcs1v15_sign_raw();
        let signature_bytes = private_key.sign(padding, &hashed_message).expect("Не удалось подписать сообщение");

        // Кодируем подпись в Base64
        self.signature = STANDARD_NO_PAD.encode(signature_bytes);
    }

    // Проверка подписи
    pub fn verify(&self) -> bool {
        let sender_der = self.sender.to_pkcs1_der().unwrap();
        let sender_base64 = STANDARD_NO_PAD.encode(sender_der.as_bytes());
        // Собираем только данные, которые участвуют в подписи
        let data_to_sign = format!("{}:{}:{}", sender_base64, self.message, self.transfer);
        // println!("Transaction data: {}", data_to_sign);
        let message_bytes = data_to_sign.into_bytes();

        // Хешируем данные
        let mut hasher = Sha256::new();
        hasher.update(message_bytes);
        let hashed_message = hasher.finalize();

        // Декодируем подпись из Base64
        let signature_bytes = STANDARD_NO_PAD.decode(&self.signature).expect("Ошибка декодирования подписи из Base64");

        // Проверяем подпись
        let padding = PaddingScheme::new_pkcs1v15_sign_raw();
        self.sender.verify(padding, &hashed_message, &signature_bytes).is_ok()
    }

    pub fn serialize(&self) -> SerializedTransaction {
        let sender_der = self.sender.to_pkcs1_der().unwrap();
        let sender_base64 = STANDARD_NO_PAD.encode(sender_der.as_bytes());

        let buyer_der = self.buyer.to_pkcs1_der().unwrap();
        let buyer_base64 = STANDARD_NO_PAD.encode(buyer_der.as_bytes());

        let seller_der = self.buyer.to_pkcs1_der().unwrap();
        let seller_base64 = STANDARD_NO_PAD.encode(seller_der.as_bytes());

        SerializedTransaction {
            sender: sender_base64,
            seller: seller_base64,
            buyer:buyer_base64,
            message: self.message.clone(),
            transfer: self.transfer,
            signature: self.signature.clone(),
        }
    }

    pub fn deserialize(serialized_transaction: SerializedTransaction) -> Result<Self, String> {
        let sender_base64 = serialized_transaction.sender;
        let sender = RsaPublicKey::from_pkcs1_der(
            &STANDARD_NO_PAD.decode(&sender_base64).unwrap()
        ).expect("Ошибка чтения ключа отправителя");

        let buyer_base64 = serialized_transaction.buyer;
        let buyer = RsaPublicKey::from_pkcs1_der(
            &STANDARD_NO_PAD.decode(&buyer_base64).unwrap()
        ).expect("Ошибка чтения ключа покупателя");

        let seller_base64 = serialized_transaction.seller;
        let seller = RsaPublicKey::from_pkcs1_der(
            &STANDARD_NO_PAD.decode(&seller_base64).unwrap()
        ).expect("Ошибка чтения ключа продавца");

        Ok(Transaction {
            sender,
            buyer,
            seller,
            message: serialized_transaction.message,
            transfer: serialized_transaction.transfer,
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


    // Получение публичного ключа отправителя
    pub fn get_sender(&self) -> RsaPublicKey {
        self.sender.clone()
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

        write!(f, "{}:{}", sender_pem, self.message)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SerializedTransaction {
    pub sender: String,
    pub buyer: String,
    pub seller: String,
    pub message: String,
    pub transfer: f64,
    pub signature: String,
}

impl SerializedTransaction {
    pub fn new(sender_base64: String, seller_base64: String, buyer_base64: String, message: String, transfer: f64) -> SerializedTransaction {
        let sender_base64 = sender_base64.trim().to_string();
        let seller_base64 = seller_base64.trim().to_string();
        let buyer_base64 = buyer_base64.trim().to_string();

        SerializedTransaction {
            //time
            sender: sender_base64,
            buyer: buyer_base64,
            seller: seller_base64,
            message,
            transfer,
            signature: "".to_string(),
        }
    }

    pub fn get_sender(&self) -> String {
        self.sender.clone()
    }

    pub fn get_transfer(&self) -> f64 {
        self.transfer
    }
}

impl Eq for SerializedTransaction {}

impl PartialEq for SerializedTransaction {
    fn eq(&self, other: &Self) -> bool {
            self.sender == other.sender &&
            self.message == other.message &&
            self.transfer == other.transfer &&
            self.signature == other.signature
    }
}

// Реализуем Ord для сортировки по приоритету
impl Ord for SerializedTransaction {
    fn cmp(&self, other: &Self) -> Ordering {
        self.transfer.partial_cmp(&other.transfer).unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for SerializedTransaction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}




#[cfg(test)]
mod transactionTests {
    use base64::Engine;
    use super::*;
    use rand::rngs::OsRng;
    use rsa::{RsaPrivateKey, pkcs1::EncodeRsaPublicKey, RsaPublicKey};

    fn generate_keys() -> (RsaPrivateKey, RsaPublicKey) {
        let mut rng = OsRng;
        let private_key = RsaPrivateKey::new(&mut rng, 2048).expect("не удалось сгенерировать ключ");
        let public_key = private_key.to_public_key();
        (private_key, public_key)
    }

    #[test]
    fn test_transaction_sign_and_verify() {
        let (sender_priv, sender_pub) = generate_keys();
        let (_, buyer_pub) = generate_keys();
        let (_, seller_pub) = generate_keys();

        let sender_b64 = base64::engine::general_purpose::STANDARD_NO_PAD.encode(sender_pub.to_pkcs1_der().unwrap());
        let buyer_b64 = base64::engine::general_purpose::STANDARD_NO_PAD.encode(buyer_pub.to_pkcs1_der().unwrap());
        let seller_b64 = base64::engine::general_purpose::STANDARD_NO_PAD.encode(seller_pub.to_pkcs1_der().unwrap());

        let mut tx = Transaction::new(sender_b64, seller_b64, buyer_b64, "Test Message".to_string(), 42.0);
        tx.sign(sender_priv.clone());

        assert!(tx.verify(), "Подпись не прошла проверку!");
    }

    #[test]
    fn test_serialize_deserialize() {
        let (sender_priv, sender_pub) = generate_keys();
        let (_, buyer_pub) = generate_keys();
        let (_, seller_pub) = generate_keys();

        let sender_b64 = base64::engine::general_purpose::STANDARD_NO_PAD.encode(sender_pub.to_pkcs1_der().unwrap());
        let buyer_b64 = base64::engine::general_purpose::STANDARD_NO_PAD.encode(buyer_pub.to_pkcs1_der().unwrap());
        let seller_b64 = base64::engine::general_purpose::STANDARD_NO_PAD.encode(seller_pub.to_pkcs1_der().unwrap());

        let mut tx = Transaction::new(sender_b64.clone(), seller_b64.clone(), buyer_b64.clone(), "Hello".to_string(), 100.0);
        tx.sign(sender_priv);

        let json = tx.to_json();
        let deserialized = Transaction::from_json(&json).unwrap();

        assert_eq!(deserialized.get_message(), "Hello");
        assert_eq!(deserialized.get_transfer(), 100.0);
        assert!(deserialized.verify());
    }

    #[test]
    fn test_transaction_ordering() {
        let tx1 = SerializedTransaction::new("sender".into(), "seller".into(), "buyer".into(), "msg".into(), 50.0);
        let tx2 = SerializedTransaction::new("sender".into(), "seller".into(), "buyer".into(), "msg".into(), 100.0);

        assert!(tx1 < tx2);
        assert!(tx2 > tx1);
    }

    #[test]
    fn test_transaction_display() {
        let (_, pub_key) = generate_keys();
        let key_b64 = base64::engine::general_purpose::STANDARD_NO_PAD.encode(pub_key.to_pkcs1_der().unwrap());

        let mut tx = Transaction::new(key_b64.clone(), key_b64.clone(), key_b64.clone(), "Display test".to_string(), 12.0);
        let display_output = format!("{}", tx);

        assert!(display_output.contains("Display test"));
    }

    #[test]
    fn test_transaction_json_roundtrip() {
        let (sender_priv, sender_pub) = generate_keys();
        let (_, buyer_pub) = generate_keys();
        let (_, seller_pub) = generate_keys();

        let sender_b64 = base64::engine::general_purpose::STANDARD_NO_PAD.encode(sender_pub.to_pkcs1_der().unwrap());
        let buyer_b64 = base64::engine::general_purpose::STANDARD_NO_PAD.encode(buyer_pub.to_pkcs1_der().unwrap());
        let seller_b64 = base64::engine::general_purpose::STANDARD_NO_PAD.encode(seller_pub.to_pkcs1_der().unwrap());

        let mut tx = Transaction::new(sender_b64.clone(), seller_b64.clone(), buyer_b64.clone(), "JSON Test".to_string(), 77.7);
        tx.sign(sender_priv);

        let json = tx.to_json();
        let restored = Transaction::from_json(&json).unwrap();

        assert_eq!(tx.get_transfer(), restored.get_transfer());
        assert_eq!(tx.get_message(), restored.get_message());
        assert!(restored.verify());
    }
}