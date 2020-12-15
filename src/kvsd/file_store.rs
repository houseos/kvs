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
use crate::grpc::kvs_api::KeyValuePair;
use crate::utils::crypto::{json_decrypt, json_encrypt};
use crate::utils::filesystem_wrapper::{
    read_persistent_store_file_to_string, write_persistent_store_file_from_string,
};

// Available Actions
pub const ACTION_STORE: u8 = 0;
pub const ACTION_DELETE: u8 = 1;

// Action for the two_lock_queue
pub struct QueueAction {
    pub kv: KeyValuePair,
    pub action: u8,
}

// Map Struct containing the HashMap of all entries
#[derive(Deserialize, Serialize)]
struct KeyValueMap {
    elements: HashMap<String, String>,
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
            println!(
                "Storing key \"{}\" with value \"{}\".",
                action.kv.key, action.kv.value
            );
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
    STORE
        .write()
        .unwrap()
        .elements
        .insert(action.kv.key, json_encrypt(action.kv.value));
    let j = match serde_json::to_string(&STORE.write().unwrap().elements) {
        Ok(j) => j,
        Err(_e) => return eprintln!("Error serializing hashmap."),
    };
    write_persistent_store_file_from_string(path, j.as_bytes());
}

fn delete_action(action: QueueAction, path: String) {
    STORE
        .write()
        .unwrap()
        .elements
        .remove(action.kv.key.as_str());
    let j = match serde_json::to_string(&STORE.write().unwrap().elements) {
        Ok(j) => j,
        Err(_e) => return eprintln!("Error serializing hashmap."),
    };
    write_persistent_store_file_from_string(path, j.as_bytes());
}

// Reading from the HashMap is possible without the queue
pub fn get_value(key: String) -> Result<String, String> {
    match STORE.read().unwrap().elements.get(key.as_str()) {
        Some(value) => {
            let decrypted_value = json_decrypt(value.to_string());
            Ok(decrypted_value)
        }
        None => Err("Key not found!".to_string()),
    }
}

// Initializes the store from the local json file on start-up.
pub fn initialize_store_from_filesystem(path: String) -> Result<String, String> {
    if !Path::new(&format!("{}/store.json", path)).exists() {
        return Ok("No persistent file available.".to_string());
    }
    let json_string = match read_persistent_store_file_to_string(format!("{}", path)) {
        Ok(json) => json,
        Err(e) => return Err(format!("Could not read persistent data file: {}", e)),
    };
    let v: HashMap<String, String> = match serde_json::from_str(json_string.as_str()) {
        Ok(val) => val,
        Err(e) => return Err(format!("Could not parse json: {}", e).to_string()),
    };
    // for each element in array
    STORE.write().unwrap().elements = v;
    // insert element in store
    Ok("Loaded store from file.".to_string())
}
