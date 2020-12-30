/*
*  log utils module
*  SPDX-License-Identifier: MIT
*  Copyright (C) 2020 Benjamin Schilling
*/

// Rust Standard Library
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

// Constants
pub const LOG_STDOUT: u8 = 0;
pub const LOG_STDERR: u8 = 1;

pub static LOG_SILENT: AtomicBool = AtomicBool::new(false);

/// Log a message to a defined destination
///
/// # Arguments
///
/// destination: LOG_STDOUT or LOG_STDERR
/// silent: if set no log message is created
pub fn log(message: String, destination: u8) {
    if !LOG_SILENT.load(Ordering::Relaxed) {
        if destination == LOG_STDOUT {
            println!("{}", message);
        } else if destination == LOG_STDERR {
            eprintln!("{}", message);
        } else {
            println!("Unknown log destination.");
        }
    }
}

pub fn set_log_silent(value: bool) {
    LOG_SILENT.store(value, Ordering::Relaxed);
}
