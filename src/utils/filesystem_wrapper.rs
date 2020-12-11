/*
*  filesystem_wrappers utils module
*  SPDX-License-Identifier: MIT
*  Copyright (C) 2020 Benjamin Schilling
*/

use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::PathBuf;
// Get directory of executable
pub fn get_exec_dir() -> io::Result<PathBuf> {
    let mut dir = env::current_exe()?;
    dir.pop();
    Ok(dir)
}

pub fn read_persistent_store_file_to_string(path: String) -> Result<String, io::Error> {
    let content = match std::fs::read_to_string(format!("{}/store.json", path)) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed reading file.");
            return Err(e);
        }
    };
    Ok(content)
}

pub fn write_persistent_store_file_from_string(path: String, data: &[u8]) {
    let mut pos = 0;
    let mut file_buffer = match File::create(format!("{}/store.json", path)) {
        Ok(o) => o,
        Err(e) => return eprintln!("Failed creating file."),
    };
    while pos < data.len() {
        let written_bytes = match file_buffer.write(&data[pos..]) {
            Ok(o) => o,
            Err(e) => return eprintln!("Could not write to file."),
        };
        pos += written_bytes;
    }
}
