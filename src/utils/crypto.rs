/*
*  crypto utils module
*  SPDX-License-Identifier: MIT
*  Copyright (C) 2020 Benjamin Schilling
*/

// Rust Standard Library
use std::str;

//Crypto libraries
use aes_gcm_siv::aead::{generic_array::GenericArray, Aead, NewAead};
use aes_gcm_siv::Aes256GcmSiv;
use rand::{Rng, RngCore};
use sha3::{Digest, Sha3_512};

// Derivation Value length
pub const DV_LEN: usize = 32;
// AES 256 GCM Initialization Vector length in bytes according to BSI TR-02102-1 (Version 2020-1)
pub const IV_LEN: usize = 12;

#[derive(Debug)]
pub struct Credentials {
    private_key: String,
    certificate: String,
}

impl Credentials {
    pub fn new(path: String) -> Result<Credentials, Box<dyn std::error::Error>> {
        let cert = match std::fs::read_to_string(format!("{}/grpc.crt", path)) {
            Ok(c) => c,
            Err(e) => return Err(Box::new(e)),
        };
        let key = match std::fs::read_to_string(format!("{}/grpc.key", path)) {
            Ok(k) => k,
            Err(e) => return Err(Box::new(e)),
        };
        Ok(Credentials {
            private_key: key,
            certificate: cert,
        })
    }

    pub fn get_private_key(&self) -> &[u8] {
        self.private_key.as_bytes()
    }

    pub fn get_certificate(&self) -> &[u8] {
        self.certificate.as_bytes()
    }
}

#[derive(Debug)]
pub struct TrustStore {
    ca_cert: String,
}

impl TrustStore {
    pub fn new(path: String) -> Result<TrustStore, Box<dyn std::error::Error>> {
        let ca_cert = match std::fs::read_to_string(format!("{}/ca.crt", path)) {
            Ok(c) => c,
            Err(e) => return Err(Box::new(e)),
        };
        Ok(TrustStore { ca_cert })
    }

    pub fn get_trusted_certificate(&self) -> &[u8] {
        self.ca_cert.as_bytes()
    }
}

// Generate random string of length DV_LEN
pub fn generate_derivation_value() -> String {
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::thread_rng();

    // generate random string
    let derivation_value: String = (0..DV_LEN)
        .map(|_| {
            let idx = rng.gen_range(0, CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    derivation_value
}

// Generate random string of length IV_LEN
pub fn generate_initialization_vector() -> [u8; IV_LEN] {
    let mut rng = rand::thread_rng();

    // generate random string
    let mut derivation_value: [u8; IV_LEN] = [0; IV_LEN];
    rng.fill_bytes(&mut derivation_value);
    derivation_value
}

// Derive password from given string.
// TODO: Currently implemented by using a hash, later libuta will be used
pub fn derive_password(derivation_value: String) -> String {
    // create a SHA3-256 object
    let mut hasher = Sha3_512::new();

    // write input message
    hasher.update(derivation_value);

    // read hash digest
    let result = hasher.finalize();
    let mut _password = format!("{:x}", result);
    _password.truncate(32);
    _password
}

// Encrypt plaintext string using provided secret and IV
fn aes_256_gcm_siv_encrypt(secret: String, iv: Vec<u8>, plaintext: String) -> Vec<u8> {
    // load key from secret
    let key = GenericArray::from_slice(secret.as_bytes());
    // Initialize AES256GCM
    let cipher = Aes256GcmSiv::new(key);
    // Set the noce to the IV
    let nonce = GenericArray::from_slice(&iv);
    // Encrypt returning the result
    cipher
        .encrypt(nonce, plaintext.as_bytes().as_ref())
        .expect("encryption failure!")
}

// Decrypt ciphertext string using provided secret and IV
fn aes_256_gcm_siv_decrypt(secret: String, iv: Vec<u8>, ciphertext: Vec<u8>) -> Vec<u8> {
    // load key from secret
    let key = GenericArray::from_slice(secret.as_bytes());
    // Initialize AES256GCM
    let cipher = Aes256GcmSiv::new(key);
    // Set the noce to the IV
    let nonce = GenericArray::from_slice(&iv);
    // Decrypt returning the result
    cipher
        .decrypt(nonce, ciphertext.as_ref())
        .expect("decryption failure!")
}

// Encrypt function wrapper for JSON Backend
pub fn json_encrypt(plaintext: String) -> String {
    // Generate derivation value
    let derivation_value = generate_derivation_value();
    // Derive password using derivation value
    let derived_password = derive_password(derivation_value.clone());
    // Generate IV
    let iv = generate_initialization_vector();
    // Encrypt
    let ciphertext = aes_256_gcm_siv_encrypt(derived_password, iv.to_vec(), plaintext);
    // Return formatted string for storage in JSON
    format!(
        "{}${}${}",
        derivation_value,
        base64::encode(iv),
        base64::encode(ciphertext)
    )
}

// Decrypt function wrapper for JSON Backend
pub fn json_decrypt(ciphertext: String) -> String {
    // Split ciphertext to three sections
    let v: Vec<&str> = ciphertext.split('$').collect();
    // Derived password is in first section
    let derived_password = derive_password(v[0].to_string());
    // Second section contains IV
    let decoded_iv: Vec<u8> = base64::decode(v[1]).unwrap();
    // Third section contains encrypted text
    let encrypted_text = base64::decode(v[2]).unwrap();
    // Decrypt
    let plaintext = aes_256_gcm_siv_decrypt(derived_password, decoded_iv, encrypted_text);
    // Return decrypted text
    return str::from_utf8(&plaintext).unwrap().to_string();
}

// Encrypt function wrapper for File Backend
pub fn file_encrypt(plaintext: String, dv: String, iv: String) -> String {
    // derive secret
    let secret: String = derive_password(dv);
    // decode IV
    let initialization_vector: Vec<u8> = base64::decode(iv).unwrap();
    // encrypt string
    let ciphertext: Vec<u8> = aes_256_gcm_siv_encrypt(secret, initialization_vector, plaintext);
    // base64 encode Vec<u8>
    base64::encode(ciphertext)
}

// Decrypt function wrapper for File Backend
pub fn file_decrypt(base64_ciphertext: String, dv: String, iv: String) -> String {
    // base64 decode string
    let ciphertext: Vec<u8> = base64::decode(base64_ciphertext).unwrap();
    // derive secret
    let secret: String = derive_password(dv);
    // decode IV
    let initialization_vector: Vec<u8> = base64::decode(iv).unwrap();
    // decrypt Vec<8u>
    let plaintext: Vec<u8> = aes_256_gcm_siv_decrypt(secret, initialization_vector, ciphertext);
    // Return decrypted text
    return str::from_utf8(&plaintext).unwrap().to_string();
}

// To run these tests use: `cargo test crypto`
// Run the following to see println!()
// cargo test crypto -- --nocapture
#[cfg(test)]
mod tests {

    use super::*;

    // ============== Password derivation ===============================
    #[test]
    fn derive_password_ok() {
        assert_eq!(
            derive_password("test".to_string()),
            "9ece086e9bac491fac5c1d1046ca11d7"
        )
    }
    #[test]
    fn derive_password_length_ok() {
        assert_eq!(derive_password("test".to_string()).len(), 32)
    }

    // ============== IV generation ===============================
    #[test]
    fn generate_initialization_vector_ok() {
        assert_eq!(generate_initialization_vector().len(), IV_LEN)
    }
}
