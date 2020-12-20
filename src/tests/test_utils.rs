/*
*  Common test functions
*  SPDX-License-Identifier: MIT
*  Copyright (C) 2020 Benjamin Schilling
*/

// Rust Standard Library
use std::process::{Child, Command};
use std::{thread, time};

// File System
use std::fs;
use std::path::Path;

// Constants
const TEST_DIR_PATH: &str = "./test_temp_dir/";
// Supported backends
const BACKEND_JSON: u8 = 0;
const BACKEND_FILE: u8 = 1;

// ============== Test Utils ==============

// Create the required directory
fn init_dir(path: String) {
    match fs::create_dir_all(path.clone()) {
        Ok(_o) => println!("Created directory at {}", path),
        Err(_e) => eprintln!("Failed creating directory {}", path),
    }
}

// Depending on used backend, either delete the store.json or delete all files
fn clean_up(backend: u8, path: String) {
    if backend == BACKEND_JSON {
        // for JSON backend delete store
        let store_path: String = format!("{}/store.json", path);
        if !Path::new(&store_path).exists() {
            println!("Nothing to clean up.");
            return;
        }
        match fs::remove_file(store_path.clone()) {
            Ok(_o) => println!("Clean-up store at {} done.", store_path),
            Err(_e) => eprintln!("Cleaning up store failed."),
        }
    } else if backend == BACKEND_FILE {
        // for File backend delete meta data and files
        if !Path::new(&path).exists() {
            println!("Nothing to clean up.");
            return;
        }
        match fs::remove_dir_all(path.clone()) {
            Ok(_o) => println!("Deleted all file in {} done.", path),
            Err(_e) => eprintln!("Deleting all file in {} failed.", path),
        }
    } else {
        eprintln!("Backend unknown");
    }
}

// Run the kvsd backend
fn run_kvsd(backend: u8, path: String) -> Result<Child, ()> {
    if backend == BACKEND_JSON {
        let kvsd_process = Command::new("target/release/kvsd")
            .args(&["--backend", "json", "--path", path.as_str()])
            .spawn()
            .expect("Failed to start kvsd process.");
        return Ok(kvsd_process);
    } else if backend == BACKEND_FILE {
        let kvsd_process = Command::new("target/release/kvsd")
            .args(&["--backend", "file", "--path", path.as_str()])
            .spawn()
            .expect("Failed to start kvsd process.");
        return Ok(kvsd_process);
    } else {
        eprintln!("Invalid Backend.");
        Err(())
    }
}

// Run kvsc with the store subcommand
pub fn run_kvsc_store(key: String, value: String) -> bool {
    let status = Command::new("target/release/kvsc")
        .args(&["store", "--key", key.as_str(), "--value", value.as_str()])
        .status()
        .expect("Failed to start kvsc process.");
    if status.success() {
        true
    } else {
        false
    }
}

// Run kvsc with the get subcommand
pub fn run_kvsc_get(key: String) -> bool {
    let status = Command::new("target/release/kvsc")
        .args(&["get", "--key", key.as_str()])
        .status()
        .expect("Failed to start kvsc process.");
    if status.success() {
        true
    } else {
        false
    }
}

// Run kvsc with the delete subcommand
pub fn run_kvsc_delete(key: String) -> bool {
    let status = Command::new("target/release/kvsc")
        .args(&["delete", "--key", key.as_str()])
        .status()
        .expect("Failed to start kvsc process.");
    if status.success() {
        true
    } else {
        false
    }
}

// Initialie the kvsd with a JSON backend
pub fn init_for_json() -> Result<Child, ()> {
    init_dir(TEST_DIR_PATH.to_string());
    // only deletes file, so directory is created first
    clean_up(BACKEND_JSON, TEST_DIR_PATH.to_string());
    let child = run_kvsd(BACKEND_JSON, TEST_DIR_PATH.to_string());
    let sleep_time = time::Duration::from_millis(1000);
    thread::sleep(sleep_time);
    return child;
}

// Initialize the kvsd with a file backend
pub fn init_for_file() -> Result<Child, ()> {
    // deletes directory, clean up is called before creating directory
    clean_up(BACKEND_FILE, TEST_DIR_PATH.to_string());
    init_dir(TEST_DIR_PATH.to_string());
    let child = run_kvsd(BACKEND_FILE, TEST_DIR_PATH.to_string());
    let sleep_time = time::Duration::from_millis(1000);
    thread::sleep(sleep_time);
    return child;
}

// Add a defined number of entries to the store
// the keys follow the format key_<number> for easy retrival
// the size specifies the number of characters of each entry
pub fn add_entries(number_of_entries: u16, size: u16) {
    for x in 0..number_of_entries {
        let mut value: String = String::new();
        for _s in 0..size {
            value = value + "a";
        }
        run_kvsc_store(format!("key_{}", x), value);
    }
}
