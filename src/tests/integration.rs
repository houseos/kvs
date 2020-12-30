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

    // Rust Standard Library
    use std::io::prelude::*;
    use std::process::{Command, Stdio};

    // File System
    use file_diff::diff_files;
    use std::fs::File;

    use crate::test_utils::*;

    // ============== Basic Functionality JSON Backend ==============
    // This sections contains end to end tests that verify specific
    // JSON backend behaviour when using kvsc and kvsd

    // Test all kvsc subcommands
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
        let mut _result: bool = false;
        // Store key
        _result = run_kvsc_store(key.clone(), value);
        // Get key
        _result = run_kvsc_get(key.clone());
        // Delete key
        _result = run_kvsc_delete(key.clone());
        // Kill kvsd
        kvsd_process.kill().expect("command wasn't running");
        assert_eq!(_result, true);
    }
    // ============== Basic Functionality File Backend ==============
    // This sections contains end to end tests that verify specific
    // file backend behaviour when using kvsc and kvsd

    // Test all kvsc subcommands
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
        let mut _result: bool = false;
        // Store key
        _result = run_kvsc_store(key.clone(), value);
        // Get key
        _result = run_kvsc_get(key.clone());
        // Delete key
        _result = run_kvsc_delete(key.clone());
        // Kill kvsd
        kvsd_process.kill().expect("command wasn't running");
        assert_eq!(_result, true);
    }
    // ============== Client Tests ==============
    // This section contains test that verify specific kvsc behaviour

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

    // Test piping a file in kvsc store, kvsc getting it and writing it to the filesystem again.
    // Afterwards the original and retrieved file have to be identical.
    #[test]
    fn integration_client_pipe_file() {
        if cfg!(target_os = "windows") {
            println!("This test can be executed under linux only, it uses bash commands.");
            return;
        }
        //start clean kvsd
        let mut kvsd_process = match init_for_file() {
            Ok(child) => child,
            Err(()) => return,
        };
        let mut _result: bool = false;
        _result = run_kvsc_store_from_file(
            "test_config_file_ini".to_string(),
            "src/tests/data/test_config_file.ini".to_string(),
        );
        // If storing it was successful, retrieve the value and write it to file
        println!("Retrieve file and store it in test_temp_dir/retrieved_config.ini.");
        if _result {
            // Get value using kvsd
            let kvsc_child = Command::new("target/release/kvsc")
                .args(&["get", "--key", "test_config_file_ini"])
                .stdout(Stdio::piped())
                .spawn()
                .expect("Failed to start kvsc process.");
            let kvsc_out = kvsc_child.stdout.expect("Failed to open cat stdout");
            // base64 decode and pipe into file

            let base64_child = Command::new("base64")
                .arg("-d")
                .stdin(Stdio::from(kvsc_out))
                .stdout(Stdio::piped())
                .spawn()
                .expect("Failed to run \"base64\" process");
            let mut base64_out = base64_child.stdout.expect("Failed to open base64 stdout");
            // compare files
            let mut _f = match File::create("test_temp_dir/retrieved_config.ini") {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Error creating file: {}", e);
                    return;
                }
            };
            let mut buff = Vec::new();
            match base64_out.read_to_end(&mut buff) {
                Ok(_o) => (),
                Err(e) => {
                    eprintln!("Error reading buffer from base64 output: {}", e);
                    return;
                }
            }
            match _f.write_all(&buff) {
                Ok(_o) => (),
                Err(e) => {
                    eprintln!("Error writing to file: {}", e);
                    return;
                }
            }
        }
        println!("Comparing retrieved file to original:");
        // Compare files
        let mut original = match File::open("src/tests/data/test_config_file.ini") {
            Ok(f) => f,
            Err(e) => panic!("{}", e),
        };
        let mut retrieved = match File::open("test_temp_dir/retrieved_config.ini") {
            Ok(f) => f,
            Err(e) => panic!("{}", e),
        };
        // Kill kvsd
        kvsd_process.kill().expect("command wasn't running");
        assert_eq!(diff_files(&mut original, &mut retrieved), true);
    }
}
