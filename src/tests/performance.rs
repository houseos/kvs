/*
*  Performance tests for kvs
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

  use rand::Rng;
  use std::fs::File;
  use std::io::prelude::*;
  use std::time::Instant;

  use crate::test_utils::*;
  // ============== Load Tests JSON Backend ==============

  // Test the maximum amount of entries in a JSON store
  #[test]
  fn performance_json_max_entries() {
    let instant_overall = Instant::now();
    let mut kvsd_process = match init_for_json() {
      Ok(child) => child,
      Err(()) => return,
    };
    // Result
    let mut _result: bool = false;

    println!("Adding 10.000 entries, this may take a while.");
    // Add 10.000 entries
    for x in 0..10000 {
      // Key Value Pair
      let mut key: String = "testkey".to_string();
      let value: String = "testvalue".to_string();
      key = key + &format!("{}", x);
      // Store key
      _result = run_kvsc_store(key.clone(), value);
      // Check that all additions were successfull
      if _result == false {
        println!("Failed adding key: {}", key.clone());
      }
    }

    // Check that 10.001th entry fails
    _result = run_kvsc_store("key10001".to_string(), "value".to_string());
    // Kill kvsd
    kvsd_process.kill().expect("command wasn't running");
    println!("Test took: {:?}", instant_overall.elapsed());
    assert_eq!(_result, false);
  }
  // Test the max size of a JSON store entry
  #[test]
  fn performance_json_max_value_length() {
    let mut kvsd_process = match init_for_json() {
      Ok(child) => child,
      Err(()) => return,
    };
    // Result
    let mut _result: bool = false;

    // Key
    let key: String = "testkey".to_string();
    // Add 1024 characters to value
    let mut value: String = String::new();
    for _x in 0..1024 {
      value = value + "a";
    }
    value = value + "b";
    // Store key value pair, it should fail
    _result = run_kvsc_store(key.clone(), value.clone());

    // Kill kvsd
    kvsd_process.kill().expect("command wasn't running");
    assert_eq!(_result, false);
  }
  // Test the performance of a JSON store with maximal size
  #[test]
  fn performance_json() {
    let instant_overall = Instant::now();
    let mut kvsd_process = match init_for_json() {
      Ok(child) => child,
      Err(()) => return,
    };
    // Result
    let mut _result: bool = false;

    println!("Adding 10.000 entries, this may take a while.");
    // Add 10.000 entries of length 1024
    for x in 0..10000 {
      // Key Value Pair
      let mut key: String = "testkey".to_string();
      let mut value: String = String::new();
      for _y in 0..1024 {
        value = value + "a";
      }
      key = key + &format!("{}", x);
      // Store key
      _result = run_kvsc_store(key.clone(), value);
      // Check that all additions were successfull
      if _result == false {
        println!("Failed adding key: {}", key.clone());
      }
    }

    // test 10 random entries in the store
    let mut rng = rand::thread_rng();
    for _x in 0..9 {
      // Start timer
      let instant = Instant::now();
      // retrieve value
      let key: String = "testkey".to_string() + &format!("{}", rng.gen_range(0, 9999));
      _result = run_kvsc_get(key.clone());
      //calculate and print diff
      println!("{:?}", instant.elapsed());
    }

    // Kill kvsd
    kvsd_process.kill().expect("command wasn't running");
    println!("Test took: {:?}", instant_overall.elapsed());
  }

  // ============== Load Tests FILE Backend ==============
  // Test the size boundary of the file store

  // Test the performance of the file store with 10 entries (each 1 kilobyte)
  // Number of files should not matter since they are not stored in RAM
  #[test]
  fn performance_file_1kb() {
    if cfg!(target_os = "windows") {
      println!("This test can be executed under linux only, it uses bash commands.");
      return;
    }
    let instant_overall = Instant::now();
    let mut kvsd_process = match init_for_file() {
      Ok(child) => child,
      Err(()) => return,
    };
    // Result
    let mut _result: bool = false;

    // create file with 1kB size
    let mut value: String = String::new();
    for _y in 0..1024 {
      value = value + "a";
    }
    let mut file = match File::create("test_temp_dir/input.txt") {
      Ok(o) => o,
      Err(_e) => return,
    };
    match file.write_all(value.as_bytes()) {
      Ok(_o) => println!("Created 1kB file"),
      Err(e) => println!("Failed creating file: {}", e),
    };

    // Add 10 entries with file as value via pipe
    for x in 0..10 {
      // Key Value Pair
      let mut key: String = "testkey".to_string();
      key = key + &format!("{}", x);
      // Store key
      _result = run_kvsc_store_from_file(key.clone(), "test_temp_dir/input.txt".to_string());
      // Check that all additions were successfull
      if _result == false {
        println!("Failed adding key: {}", key.clone());
      }
    }

    // test 10 random entries in the store
    let mut rng = rand::thread_rng();
    for _x in 0..9 {
      // Start timer
      let instant = Instant::now();
      // retrieve value
      let key: String = "testkey".to_string() + &format!("{}", rng.gen_range(0, 10));
      _result = run_kvsc_get(key.clone());
      //calculate and print diff
      println!("{:?}", instant.elapsed());
      if !_result {
        println!("Key {} did not exist, failed!", key.clone());
      }
    }

    // Kill kvsd
    kvsd_process.kill().expect("command wasn't running");
    println!("Test took: {:?}", instant_overall.elapsed());
  }

  // Test the performance of the file store with 10 entries (each 1 megabyte)
  // Number of files should not matter since they are not stored in RAM
  #[test]
  fn performance_file_1mb() {
    if cfg!(target_os = "windows") {
      println!("This test can be executed under linux only, it uses bash commands.");
      return;
    }
    let instant_overall = Instant::now();
    let mut kvsd_process = match init_for_file() {
      Ok(child) => child,
      Err(()) => return,
    };

    // create file with 1MB size
    let mut value: String = String::new();
    for _y in 0..1048576 {
      value = value + "a";
    }
    let mut file = match File::create("test_temp_dir/input.txt") {
      Ok(o) => o,
      Err(_e) => return,
    };
    match file.write_all(value.as_bytes()) {
      Ok(_o) => println!("Created 1MB file"),
      Err(e) => println!("Failed creating file: {}", e),
    };
    println!("1");
    // Result
    let mut _result: bool = false;
    // Add 10 entries via pipe
    for x in 0..10 {
      // Key Value Pair
      let mut key: String = "testkey".to_string();
      key = key + &format!("{}", x);
      // Store key
      _result = run_kvsc_store_from_file(key.clone(), "test_temp_dir/input.txt".to_string());
      // Check that all additions were successfull
      if _result == false {
        println!("Failed adding key: {}", key.clone());
      }
    }

    println!("2");
    // test 10 random entries in the store
    let mut rng = rand::thread_rng();
    for _x in 0..9 {
      // Start timer
      let instant = Instant::now();
      // retrieve value
      let key: String = "testkey".to_string() + &format!("{}", rng.gen_range(0, 10));
      _result = run_kvsc_get(key.clone());
      //calculate and print diff
      println!("{:?}", instant.elapsed());
      if !_result {
        println!("Key {} did not exist, failed!", key.clone());
      }
    }

    println!("3");
    // Kill kvsd
    kvsd_process.kill().expect("command wasn't running");
    println!("Test took: {:?}", instant_overall.elapsed());
  }
  // Test the performance of the file store with 10.000 entries (each biggest size)
}
