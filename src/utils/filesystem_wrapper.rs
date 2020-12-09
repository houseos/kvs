/*
*  filesystem_wrappers utils module
*  SPDX-License-Identifier: MIT
*  Copyright (C) 2020 Benjamin Schilling
*/

use std::env;
use std::io;
use std::path::PathBuf;

// Get directory of executable
pub fn get_exec_dir() -> io::Result<PathBuf> {
    let mut dir = env::current_exe()?;
    dir.pop();
    Ok(dir)
}
