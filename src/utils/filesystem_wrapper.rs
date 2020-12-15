/*
*  filesystem_wrappers utils module
*  SPDX-License-Identifier: MIT
*  Copyright (C) 2020 Benjamin Schilling
*/

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

pub fn read_persistent_store_file_to_string(path: String) -> Result<String, io::Error> {
    let content = match std::fs::read_to_string(format!("{}/store.json", path)) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed reading file: {}", e);
            return Err(e);
        }
    };
    Ok(content)
}

pub fn write_persistent_store_file_from_string(path: String, data: &[u8]) {
    let mut pos = 0;
    let mut file_buffer = match File::create(format!("{}/store.json", path)) {
        Ok(o) => o,
        Err(e) => return eprintln!("Failed creating file: {}", e),
    };
    while pos < data.len() {
        let written_bytes = match file_buffer.write(&data[pos..]) {
            Ok(o) => o,
            Err(e) => return eprintln!("Could not write to file: {}", e),
        };
        pos += written_bytes;
    }
}
