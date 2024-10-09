use std::io;
use std::io::Write;
use crate::coin::blockchain::wallet::Wallet;

mod coin;

fn get_input_text(info_text: &str) -> String {
    print!("{}: ", info_text);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => input.trim().to_string(),
        Err(e) => {
            eprintln!("Error reading input: {}", e);
            String::new()
        }
    }
}

fn main() {
    let wallet = Wallet::load_from_file("cache/wallet.json");
    let public_key_pem = wallet.get_public_key_pem();
    let private_key = wallet.get_private_key_pem();
    println!("Public wallet key: {}, {}", public_key_pem, private_key);
}