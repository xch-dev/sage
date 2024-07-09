use bip39::Mnemonic;
use error::Result;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use tauri::command;

mod error;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![generate_mnemonic])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[command]
fn generate_mnemonic(use_24_words: bool) -> Result<String> {
    let mut rng = ChaCha20Rng::from_entropy();
    let mnemonic = if use_24_words {
        let entropy: [u8; 32] = rng.gen();
        Mnemonic::from_entropy(&entropy)?
    } else {
        let entropy: [u8; 16] = rng.gen();
        Mnemonic::from_entropy(&entropy)?
    };
    Ok(mnemonic.to_string())
}
