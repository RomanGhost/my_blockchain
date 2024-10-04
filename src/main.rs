use rsa::{PublicKey, RsaPrivateKey, RsaPublicKey, PaddingScheme};
use rand::rngs::OsRng;
use rsa::pkcs8::EncodePublicKey;
// используем надежный генератор случайных чисел

fn generate_keys() -> (RsaPrivateKey, RsaPublicKey) {
    // генерируем 2048-битный приватный ключ
    let mut rng = OsRng;
    let private_key = RsaPrivateKey::new(&mut rng, 256)
        .expect("Ошибка генерации ключа");

    // получаем публичный ключ из приватного
    let public_key = RsaPublicKey::from(&private_key);

    (private_key, public_key)
}

fn encrypt_message(public_key: &RsaPublicKey, message: &[u8]) -> Vec<u8> {
    let mut rng = OsRng;
    let padding = PaddingScheme::new_pkcs1v15_encrypt(); // используем PKCS1 v1.5 для шифрования

    // шифруем сообщение
    public_key
        .encrypt(&mut rng, padding, message)
        .expect("Ошибка шифрования")
}

fn decrypt_message(private_key: &RsaPrivateKey, encrypted_data: &[u8]) -> Vec<u8> {
    let padding = PaddingScheme::new_pkcs1v15_encrypt();

    // дешифруем сообщение
    private_key
        .decrypt(padding, encrypted_data)
        .expect("Ошибка дешифрования")
}

fn main() {
    // генерируем ключи
    let (private_key, public_key) = generate_keys();

    // исходное сообщение
    let message = "Привет, Хабр!".to_string();
    let message = message.into_bytes();

    // зашифровываем сообщение
    let encrypted_data = encrypt_message(&public_key, &message);
    println!("{:?}", public_key.to_public_key_der().unwrap());

    // дешифровываем сообщение
    let decrypted_data = decrypt_message(&private_key, &encrypted_data);

    println!("Расшифрованное сообщение: {:?}", String::from_utf8_lossy(&decrypted_data));
}

