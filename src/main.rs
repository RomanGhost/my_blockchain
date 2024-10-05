use rsa::{RsaPrivateKey, RsaPublicKey, PaddingScheme, PublicKey};
use sha2::Sha256;
use rand::rngs::OsRng;
use rsa::signature::digest::Digest;

fn main() {
    // Генерация пары ключей
    let mut rng = OsRng;
    let bits = 2048;
    let private_key = RsaPrivateKey::new(&mut rng, bits).expect("Не удалось сгенерировать ключ");
    let public_key = RsaPublicKey::from(&private_key);

    // Исходное сообщение
    let message = "Сообщение для подписи".to_string();
    let message = message.into_bytes();

    // Хеширование сообщения
    let mut hasher = Sha256::new();
    hasher.update(message);
    let hashed_message = hasher.finalize();

    // Подпись хеша
    let padding = PaddingScheme::new_pkcs1v15_sign_raw();
    let signature = private_key.sign(padding, &hashed_message).expect("Не удалось подписать сообщение");

    // Проверка подписи
    let padding = PaddingScheme::new_pkcs1v15_sign_raw();
    let is_valid = public_key.verify(padding, &hashed_message, &signature).is_ok();

    // Результат проверки
    if is_valid {
        println!("Подпись верна!");
    } else {
        println!("Подпись неверна!");
    }
}
