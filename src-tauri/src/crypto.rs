use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use keyring::Entry;
use rand::{rngs::OsRng, RngCore};

const APP_NAME: &str = "nethopper";
const KEY_NAME: &str = "master-key";

fn get_or_create_key() -> Result<Vec<u8>, String> {
    let entry = Entry::new(APP_NAME, KEY_NAME).map_err(|e| e.to_string())?;

    match entry.get_password() {
        Ok(key_b64) => B64.decode(key_b64).map_err(|e| e.to_string()),
        Err(_) => {
            let mut key = vec![0u8; 32];
            OsRng.fill_bytes(&mut key);
            entry
                .set_password(&B64.encode(&key))
                .map_err(|e| e.to_string())?;
            Ok(key)
        }
    }
}

pub fn encrypt(plaintext: &str) -> Result<String, String> {
    let key_bytes = get_or_create_key()?;
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| e.to_string())?;

    // Gabung nonce + ciphertext lalu encode base64
    let mut combined = nonce_bytes.to_vec();
    combined.extend(ciphertext);

    Ok(B64.encode(combined))
}

pub fn decrypt(encrypted: &str) -> Result<String, String> {
    let key_bytes = get_or_create_key()?;
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    let combined = B64.decode(encrypted).map_err(|e| e.to_string())?;

    // Split nonce (12 bytes) dan ciphertext
    if combined.len() < 12 {
        return Err("Invalid encrypted data".to_string());
    }
    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| "Decryption failed".to_string())?;

    String::from_utf8(plaintext).map_err(|e| e.to_string())
}
