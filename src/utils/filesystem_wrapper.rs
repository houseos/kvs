/*
*  filesystem_wrappers utils module
*  SPDX-License-Identifier: MIT
*  Copyright (C) 2020 Benjamin Schilling
*/

// Rust Standard Library
use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;

// Get directory of executable
pub fn get_exec_dir() -> String {
    let mut dir = match env::current_exe() {
        Ok(dir) => dir,
        Err(_e) => {
            eprintln!("Could not access executable directory.");
            std::process::exit(0x0001);
        }
    };
    dir.pop();
    format!("{}", dir.display())
}

// Generic delete file function
pub fn delete_file(path: String) -> Result<(), io::Error> {
    match std::fs::remove_file(path.clone()) {
        Ok(_o) => Ok(()),
        Err(e) => {
            eprintln!("Failed deleting file: {}", path);
            Err(e)
        }
    }
}

// Generic read vec<u8> from file
pub fn read_file_to_vec(path: String) -> Result<Vec<u8>, io::Error> {
    let content = match std::fs::read(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed reading file: {}", e);
            return Err(e);
        }
    };
    Ok(content)
}

// Generic read string from file
pub fn read_file_to_string(path: String) -> Result<String, io::Error> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed reading file: {}", e);
            return Err(e);
        }
    };
    Ok(content)
}

// Generic write string to file
pub fn write_string_to_file(path: String, data: String) {
    let string_buffer = data.as_bytes();
    let mut pos = 0;
    let mut file_buffer = match File::create(path) {
        Ok(o) => o,
        Err(e) => return eprintln!("Failed creating file: {}", e),
    };
    while pos < data.len() {
        let written_bytes = match file_buffer.write(&string_buffer[pos..]) {
            Ok(o) => o,
            Err(e) => return eprintln!("Could not write to file: {}", e),
        };
        pos += written_bytes;
    }
}

// JSON Backend specific read file
pub fn read_persistent_store_file_to_string(path: String) -> Result<String, io::Error> {
    let content = match read_file_to_string(format!("{}/store.json", path)) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed reading JSON file: {}", e);
            return Err(e);
        }
    };
    Ok(content)
}

// JSON Backend specific write file
pub fn write_persistent_store_file_from_string(path: String, data: String) {
    write_string_to_file(format!("{}/store.json", path), data);
}
