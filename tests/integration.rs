/*
*  Integrations tests for kvs
*  SPDX-License-Identifier: MIT
*  Copyright (C) 2020 Benjamin Schilling
*/

// To run these tests use: `cargo test integration -- --test-threads=1`.
// The --test-threads=1 option is crucial, otherwise cargo tries to execute
// all tests in parallel that would result in kvsd instances trying to bind to the same port.
// Run the following to see println!():
// `cargo test integration -- --test-threads=1 --nocapture`
#[cfg(test)]
mod tests {

    use base64;
    use rand::Rng;

    // Rust Standard Library
    use std::process::{Child, Command};

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
    fn run_kvsc_store(key: String, value: String) -> bool {
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
    fn run_kvsc_get(key: String) -> bool {
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
    fn run_kvsc_delete(key: String) -> bool {
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

    fn init_for_json() -> Result<Child, ()> {
        init_dir(TEST_DIR_PATH.to_string());
        // only deletes file, so directory is created first
        clean_up(BACKEND_JSON, TEST_DIR_PATH.to_string());
        run_kvsd(BACKEND_JSON, TEST_DIR_PATH.to_string())
    }

    fn init_for_file() -> Result<Child, ()> {
        // deletes directory, clean up is called before creating directory
        clean_up(BACKEND_FILE, TEST_DIR_PATH.to_string());
        init_dir(TEST_DIR_PATH.to_string());
        run_kvsd(BACKEND_FILE, TEST_DIR_PATH.to_string())
    }

    // ============== Basic Functionality JSON Backend ==============
    #[test]
    fn integration_json_store_get_delete() {
        let mut kvsd_process = match init_for_json() {
            Ok(child) => child,
            Err(()) => return,
        };
        // Key Value Pair
        let key: String = "testkey".to_string();
        let value: String = "testvalue".to_string();

        // Result
        let mut result: bool = false;
        // Store key
        result = run_kvsc_store(key.clone(), value);
        // Get key
        result = run_kvsc_get(key.clone());
        // Delete key
        result = run_kvsc_delete(key.clone());
        // Kill kvsd
        kvsd_process.kill().expect("command wasn't running");
        assert_eq!(result, true);
    }
    // ============== Basic Functionality File Backend ==============
    #[test]
    fn integration_file_store_get_delete() {
        let mut kvsd_process = match init_for_file() {
            Ok(child) => child,
            Err(()) => return,
        };
        // Key Value Pair
        let key: String = "testkey".to_string();
        let value: String = "testvalue".to_string();

        // Result
        let mut result: bool = false;
        // Store key
        result = run_kvsc_store(key.clone(), value);
        // Get key
        result = run_kvsc_get(key.clone());
        // Delete key
        result = run_kvsc_delete(key.clone());
        // Kill kvsd
        kvsd_process.kill().expect("command wasn't running");
        assert_eq!(result, true);
    }
    // ============== Client Tests ==============

    // Tests that the client returns a failed status if the to be returned key is not found
    #[test]
    fn integration_client_get_not_found() {
        let mut kvsd_process = match init_for_json() {
            Ok(child) => child,
            Err(()) => return,
        };
        // Key
        let key: String = "testkey".to_string();
        // Get key
        let result: bool = run_kvsc_get(key.clone());
        // Kill kvsd
        kvsd_process.kill().expect("command wasn't running");
        assert_eq!(result, false);
    }
    // Tests that the client returns a failed status if the to be deleted key is not found
    #[test]
    fn integration_client_delete_not_found() {
        let mut kvsd_process = match init_for_json() {
            Ok(child) => child,
            Err(()) => return,
        };
        // Key
        let key: String = "testkey".to_string();
        // Delete key
        let result: bool = run_kvsc_delete(key.clone());
        // Kill kvsd
        kvsd_process.kill().expect("command wasn't running");
        assert_eq!(result, false);
    }
}
