use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use aes_gcm::aead::{Aead, Key};
use sha2::{Sha256, Digest};

pub struct CryptoService;

impl CryptoService {
    pub fn encrypt(plaintext: &str, secret: &str) -> String {
        let key_bytes = Sha256::digest(secret.as_bytes());
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);

        let nonce_bytes: [u8; 12] = rand::random();
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher.encrypt(nonce, plaintext.as_bytes())
            .expect("encryption failed");

        format!(
            "{}:{}",
            hex::encode(nonce_bytes),
            hex::encode(ciphertext),
        )
    }

    #[allow(dead_code)]
    pub fn decrypt(stored: &str, secret: &str) -> Option<String> {
        let parts: Vec<&str> = stored.split(':').collect();
        if parts.len() != 2 { return None; }

        let nonce_bytes = hex::decode(parts[0]).ok()?;
        let ciphertext = hex::decode(parts[1]).ok()?;

        let key_bytes = Sha256::digest(secret.as_bytes());
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);

        let nonce = Nonce::from_slice(&nonce_bytes);
        let plaintext = cipher.decrypt(nonce, ciphertext.as_slice()).ok()?;

        String::from_utf8(plaintext).ok()
    }
}
