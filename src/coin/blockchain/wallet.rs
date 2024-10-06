use rand::rngs::OsRng;
use rsa::{RsaPrivateKey, RsaPublicKey};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use rsa::pkcs1::{DecodeRsaPublicKey, EncodeRsaPublicKey, LineEnding, DecodeRsaPrivateKey};

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
        let public_key_pem = self.public_key.to_pkcs1_pem(LineEnding::LF).unwrap();  // Сериализация публичного ключа в PEM
        let private_key_pem = self.private_key.to_pkcs1_pem(LineEnding::LF).unwrap(); // Сериализация приватного ключа в PEM

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
}

// Структура для сериализованного кошелька
#[derive(Serialize, Deserialize)]
struct SerializedWallet {
    public_key: String,    // Публичный ключ в виде строки (PEM)
    private_key: String,   // Приватный ключ в виде строки (PEM)
    amount: f64,
}
