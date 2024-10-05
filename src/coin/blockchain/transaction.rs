use rsa::RsaPublicKey;
use serde::{Serialize, Deserialize};
use rsa::pkcs1::{DecodeRsaPublicKey, EncodeRsaPublicKey, LineEnding, RsaPrivateKey};

// Структура для сериализованной транзакции
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    id: u64,
    sender: String,   // Публичный ключ отправителя в строковом формате
    receiver: String, // Публичный ключ получателя в строковом формате
    message: String,
    tax: f64,
    signature: Option<Vec<u8>>,
}

impl Transaction {
    // Создание новой транзакции с конвертацией ключей в строковый формат
    pub fn new(sender: RsaPublicKey, receiver: RsaPublicKey, message: String, tax: f64, signature: Option<Vec<u8>>) -> Transaction {
        let sender_pem = sender.to_pkcs1_pem(LineEnding::LF).unwrap();  // Преобразуем публичный ключ отправителя в PEM
        let receiver_pem = receiver.to_pkcs1_pem(LineEnding::LF).unwrap(); // Преобразуем публичный ключ получателя в PEM

        Transaction {
            id: 0,
            sender: sender_pem,
            receiver: receiver_pem,
            message,
            tax,
            signature,
        }
    }

    // todo!("Сделать подпись и проверку подписи транзакции");

    // Преобразование транзакции в строку (для демонстрации)
    pub fn to_string(&self) -> String {
        format!("{}:{}:{}:{}", self.sender, self.receiver, self.message, self.tax)
    }

    // Преобразование транзакции в JSON
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    // Получение ID транзакции
    pub fn get_id(&self) -> u64 {
        self.id
    }

    // Получение публичного ключа получателя (конвертация обратно из PEM)
    pub fn get_receiver(&self) -> RsaPublicKey {
        RsaPublicKey::from_pkcs1_pem(&self.receiver).unwrap()  // Преобразуем строку обратно в RsaPublicKey
    }

    // Получение публичного ключа отправителя (конвертация обратно из PEM)
    pub fn get_sender(&self) -> RsaPublicKey {
        RsaPublicKey::from_pkcs1_pem(&self.sender).unwrap()  // Преобразуем строку обратно в RsaPublicKey
    }

    // Получение комиссии транзакции
    pub fn get_tax(&self) -> f64 {
        self.tax
    }
}
