/*
*  kvsd file store Module
*  SPDX-License-Identifier: MIT
*  Copyright (C) 2020 Benjamin Schilling
*/

// Rust Standard Library
use std::collections::HashMap;
use std::path::Path;
use std::sync::RwLock;

// lazy static
use lazy_static::lazy_static;

// json
use serde::{Deserialize, Serialize};

// kvs modules
use crate::store::store_actions::{QueueAction, ACTION_DELETE, ACTION_STORE};
use utils::crypto::{
    file_decrypt, file_encrypt, generate_derivation_value, generate_initialization_vector,
    json_decrypt, json_encrypt,
};
use utils::filesystem_wrapper::{delete_file, read_file_to_string, write_string_to_file};

// Value File Meta Data
#[derive(Deserialize, Serialize)]
struct ValueMetaData {
    filename: String,
    derivation_value: String,
    initialization_vector: String,
}

// Map Struct containing the HashMap of all entries
#[derive(Deserialize, Serialize)]
struct KeyValueMap {
    elements: HashMap<String, ValueMetaData>,
}

// Static HashMap containing all elements
lazy_static! {
    static ref STORE: RwLock<KeyValueMap> = RwLock::new(KeyValueMap {
        elements: HashMap::new(),
    });
}

// Handle a QueueAction
pub fn handle_action(action: QueueAction, path: String) {
    match action.action {
        ACTION_STORE => {
            println!("Storing key \"{}\".", action.kv.key);
            store_action(action, path);
        }
        ACTION_DELETE => {
            println!("Deleting key \"{}\".", action.kv.key);
            delete_action(action, path);
        }
        _ => {
            eprintln!("No matching action available.");
        }
    }
}

fn store_action(action: QueueAction, path: String) {
    // generate new derivation value
    let derivation_value = generate_derivation_value();
    // generate new iv
    let iv = generate_initialization_vector();
    // base64 encode IV for storage in JSON
    let base64_iv = base64::encode(iv);
    // encrypt value
    let ciphertext = file_encrypt(action.kv.value, derivation_value.clone(), base64_iv.clone());
    // if entry does not exist in hashmap
    if !STORE.read().unwrap().elements.contains_key(&action.kv.key) {
        // generate filename
        let filename = generate_derivation_value();
        // write file to filesystem
        write_string_to_file(format!("{}/{}", path, filename), ciphertext);
        // store meta data in hashmap
        STORE.write().unwrap().elements.insert(
            action.kv.key,
            ValueMetaData {
                filename,
                derivation_value,
                initialization_vector: base64_iv,
            },
        );
    } else {
        // retrieve filename from hashmap
        let filename = match STORE.read().unwrap().elements.get(action.kv.key.as_str()) {
            Some(value) => value.filename.clone(),
            None => "".to_string(),
        };
        if filename == "" {
            eprintln!(
                "Could not find key \"{}\" that should already be stored. Ommiting store action.",
                action.kv.key
            );
            return;
        }
        // store value in file with filename from hashmap
        write_string_to_file(format!("{}/{}", path, filename), ciphertext);
        // store meta data in hashmap
        STORE.write().unwrap().elements.insert(
            action.kv.key,
            ValueMetaData {
                filename,
                derivation_value,
                initialization_vector: base64_iv,
            },
        );
    }
    // serialize hashmap, encrypt it and store it
    save_meta_data_to_file(path);
}

fn delete_action(action: QueueAction, path: String) {
    // retrieve filename from hashmap
    let filename = match STORE.read().unwrap().elements.get(action.kv.key.as_str()) {
        Some(value) => value.filename.clone(),
        None => "".to_string(),
    };
    // delete file
    match delete_file(format!("{}/{}", path, filename)) {
        Ok(_o) => println!("Deleting key \"{}\".", action.kv.key),
        Err(_e) => {
            eprintln!("Could not delete key \"{}\". Keeping it.", action.kv.key);
            return;
        }
    }
    // delete entry in hashmap
    STORE
        .write()
        .unwrap()
        .elements
        .remove(action.kv.key.as_str());
    // serialize hashmap, encrypt it and store it
    save_meta_data_to_file(path);
}

// Reading from the HashMap is possible without the queue
pub fn get_value(key: String, path: String) -> Result<String, String> {
    match STORE.read().unwrap().elements.get(key.as_str()) {
        Some(_value) => {
            // retrieve filename from hashmap
            let filename = match STORE.read().unwrap().elements.get(key.as_str()) {
                Some(value) => value.filename.clone(),
                None => "".to_string(),
            };
            // retrieve dv from hashmap
            let dv = match STORE.read().unwrap().elements.get(key.as_str()) {
                Some(value) => value.derivation_value.clone(),
                None => "".to_string(),
            };
            // retrieve iv from hashmap
            let iv = match STORE.read().unwrap().elements.get(key.as_str()) {
                Some(value) => value.initialization_vector.clone(),
                None => "".to_string(),
            };

            // load encrypted file
            let base64_ciphertext = match read_file_to_string(format!("{}{}", path, filename)) {
                Ok(o) => o,
                Err(_e) => {
                    eprintln!(
                        "Could not read file \"{}\" to string to retrieve it's value.",
                        format!("{}{}", path, filename)
                    );
                    return Err("File of key not found.".to_string());
                }
            };
            // decrypt using key and iv
            let decrypted_value = file_decrypt(base64_ciphertext, dv, iv);
            Ok(decrypted_value)
        }
        None => Err("Key not found!".to_string()),
    }
}

// Initializes the store from the local json file on start-up.
pub fn load_meta_data_from_file(path: String) -> Result<String, String> {
    // Assemble file path
    let file_path: String = format!("{}/kvsd-meta-data.json", path);
    // check whether file exists
    if !Path::new(&file_path).exists() {
        return Ok("No meta-data file available.".to_string());
    }
    // load encrypted string
    let encrypted_string = match read_file_to_string(file_path) {
        Ok(json) => json,
        Err(e) => return Err(format!("Could not read stored meta-data file: {}", e)),
    };
    // Decrypt content
    let json_string = json_decrypt(encrypted_string);
    // parse decrypted string
    let v: HashMap<String, ValueMetaData> = match serde_json::from_str(json_string.as_str()) {
        Ok(val) => val,
        Err(e) => return Err(format!("Could not parse json: {}", e)),
    };
    // for each element in array
    STORE.write().unwrap().elements = v;
    // insert element in store
    Ok("Loaded stored meta-data from file.".to_string())
}

fn save_meta_data_to_file(path: String) {
    // serialize HashMap
    let json_string = match serde_json::to_string(&STORE.write().unwrap().elements) {
        Ok(j) => j,
        Err(_e) => return eprintln!("Error serializing hashmap."),
    };
    // encrypt json
    let encrypted_json = json_encrypt(json_string);
    // write to file
    write_string_to_file(format!("{}/kvsd-meta-data.json", path), encrypted_json);
}
