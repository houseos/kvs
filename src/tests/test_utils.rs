/*
*  Common test functions
*  SPDX-License-Identifier: MIT
*  Copyright (C) 2020 Benjamin Schilling
*/

// Rust Standard Library
use std::process::{Child, Command, Stdio};
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
            Ok(_o) => println!("Deleting all files in {} done.", path),
            Err(_e) => eprintln!("Deleting all files in {} failed.", path),
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

fn runs_kvsd_silent(backend: u8, path: String) -> Result<Child, ()> {
    if backend == BACKEND_JSON {
        let kvsd_process = Command::new("target/release/kvsd")
            .args(&["--silent", "--backend", "json", "--path", path.as_str()])
            .spawn()
            .expect("Failed to start kvsd process.");
        return Ok(kvsd_process);
    } else if backend == BACKEND_FILE {
        let kvsd_process = Command::new("target/release/kvsd")
            .args(&["--silent", "--backend", "file", "--path", path.as_str()])
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
        .args(&[
            "--silent",
            "store",
            "--key",
            key.as_str(),
            "--value",
            value.as_str(),
        ])
        .status()
        .expect("Failed to start kvsc process.");
    if status.success() {
        true
    } else {
        false
    }
}

pub fn run_kvsc_store_from_file(key: String, filepath: String) -> bool {
    println!("cat");
    let mut cat_child = Command::new("cat")
        .arg(filepath)
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to run \"cat\" process");
    println!("cat before wait");
    match cat_child.wait() {
        Ok(_o) => println!("cat successful"),
        Err(e) => println!("cat failed: {}", e),
    };
    println!("cat after wait");
    let cat_out = cat_child.stdout.expect("Failed to open cat stdout");
    // pipe output to base64 command
    println!("base64");
    let mut base64_child = Command::new("base64")
        // .arg("-e")
        .stdin(Stdio::from(cat_out))
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to run \"base64\" process");
    match base64_child.wait() {
        Ok(_o) => println!("base64 successful"),
        Err(e) => println!("base64 failed: {}", e),
    };
    let base64_out = base64_child.stdout.expect("Failed to open base64 stdout");
    // remove possible new line characters
    println!("tr");
    let mut tr_child = Command::new("tr")
        .arg("-d")
        .arg("'\n'")
        .stdin(Stdio::from(base64_out))
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to run \"base64\" process");
    match tr_child.wait() {
        Ok(_o) => println!("tr successful"),
        Err(e) => println!("tr failed: {}", e),
    };
    let tr_out = tr_child.stdout.expect("Failed to open tr stdout");
    // pipe returned value to kvsc
    let mut _result: bool = false;
    println!("kvsc");
    let status = Command::new("target/release/kvsc")
        .args(&["--silent", "store", "--key", &key, "--pipe"])
        .stdin(Stdio::from(tr_out))
        .status()
        .expect("Failed to start kvsc process.");
    if status.success() {
        return true;
    } else {
        return false;
    }
}

// Run kvsc with the get subcommand
pub fn run_kvsc_get(key: String) -> bool {
    let status = Command::new("target/release/kvsc")
        .args(&["--silent", "get", "--key", key.as_str()])
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
        .args(&["--silent", "delete", "--key", key.as_str()])
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
    let child = runs_kvsd_silent(BACKEND_JSON, TEST_DIR_PATH.to_string());
    let sleep_time = time::Duration::from_millis(1000);
    thread::sleep(sleep_time);
    return child;
}

// Initialize the kvsd with a file backend
pub fn init_for_file() -> Result<Child, ()> {
    // deletes directory, clean up is called before creating directory
    clean_up(BACKEND_FILE, TEST_DIR_PATH.to_string());
    init_dir(TEST_DIR_PATH.to_string());
    let child = runs_kvsd_silent(BACKEND_FILE, TEST_DIR_PATH.to_string());
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
