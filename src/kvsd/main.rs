/*
*  kvsd main
*  SPDX-License-Identifier: MIT
*  Copyright (C) 2020 Benjamin Schilling
*/

#![feature(proc_macro_hygiene, decl_macro)]

// Rust Standard Library
use std::thread;

// Store
extern crate two_lock_queue;

//kvs modules
mod grpc;
mod store;
use crate::utils::filesystem_wrapper;
use crate::utils::input_validation;
use store::file_store;
use store::json_store;
use store::store_actions::QueueAction;
use utils;

// CLI interface
extern crate clap;
use clap::{App, Arg};

// CLI Signal handling
extern crate ctrlc;

// Supported backends
const BACKEND_JSON: u8 = 0;
const BACKEND_FILE: u8 = 1;

fn main() {
    // Properly handle CTRL-C signals
    ctrlc::set_handler(move || {
        println!("Received Ctrl+C, shutting down.");
        std::process::exit(0x0000);
    })
    .expect("Error setting Ctrl+C handler");

    // Specify commandline arguments
    let matches = App::new("kvsd")
        .version(clap::crate_version!())
        .author("Benjamin Schilling <benjamin.schilling33@gmail.com>")
        .arg(
            Arg::with_name("ip")
                .help("IP address the kvs daemon shall bind the gRPC interface to.")
                .required(false)
                .long("ip")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("port")
                .help("Port the kvs daemon shall bind the gRPC interface to.")
                .required(false)
                .long("port")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("path")
            .help("Filesystem path for the persistent store.")
            .required(false)
            .long("path")
            .takes_value(true),
        )
        .arg(
            Arg::with_name("backend")
            .help("Backend to be used, either \"json\" or \"file\".")
            .required(false)
            .long("backend")
            .takes_value(true)
            .possible_values(&["json", "file"]),
        )
        .arg(
            Arg::with_name("tls")
                .help("Set to enable TLS support for gRPC.\nIf set certificate and private key are expected as grpc.crt\nand grpc.key in the execution directory of kvsc binary.")
                .long("tls"),
        )
        .get_matches();
    // Set IP and Port to default values
    let mut ip: String = "127.0.0.1".to_string();
    // Set IP to provided value if existing
    if matches.is_present("ip") {
        if input_validation::validate_ipv4(matches.value_of("ip").unwrap().to_string()) {
            ip = matches.value_of("ip").unwrap().to_string();
        } else {
            eprintln!(
                "IP parameter \"{}\" invalid, only IPv4 allowed.",
                matches.value_of("ip").unwrap().to_string()
            );
            std::process::exit(0x0001);
        }
    }
    // Set Port to provided value if existing
    let mut port: String = "27001".to_string();
    if matches.is_present("port") {
        if input_validation::validate_port(matches.value_of("port").unwrap().to_string()) {
            port = matches.value_of("port").unwrap().to_string();
        } else {
            eprintln!(
                "Port parameter \"{}\" invalid, only valid TCP port numbers allowed.",
                matches.value_of("port").unwrap().to_string()
            );
            std::process::exit(0x0001);
        }
    }
    // Set path for persistent store file
    let mut path: String = filesystem_wrapper::get_exec_dir();
    if matches.is_present("path") {
        if input_validation::validate_path(matches.value_of("path").unwrap().to_string()) {
            path = matches.value_of("path").unwrap().to_string();
        } else {
            eprintln!(
                "Path parameter \"{}\" invalid, only valid filesystem paths using alphanumeric characters, \"\\\", \"/\", \".\", \":\", \"-\", \"_\" are allowed.",
                matches.value_of("Path").unwrap().to_string()
            );
            std::process::exit(0x0001);
        }
    }
    // Set backend, json is default
    let mut backend: u8 = BACKEND_JSON;

    if matches.is_present("backend") {
        let backend_value = matches.value_of("backend").unwrap();
        if backend_value == "file" {
            backend = BACKEND_FILE;
        }
    }

    // Read persistent store from file
    if backend == BACKEND_JSON {
        match json_store::initialize_store_from_file(path.clone()) {
            Ok(ok) => println!("Finished loading file: {}", ok),
            Err(e) => eprintln!("Error loading file: {}", e),
        }
    } else if backend == BACKEND_FILE {
        match file_store::load_meta_data_from_file(path.clone()) {
            Ok(ok) => println!("Finished loading file: {}", ok),
            Err(e) => eprintln!("Error loading file: {}", e),
        }
    }

    let (tx, rx) = two_lock_queue::unbounded::<QueueAction>();

    // Start the gRPC Server in a thread
    let grpc_path = path.clone();
    thread::spawn(move || {
        match grpc::start_grpc_server(
            ip,
            port,
            matches.is_present("tls"),
            tx.clone(),
            backend,
            grpc_path,
        ) {
            Ok(o) => {
                println!("{:?}", o);
            }
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(0x0001);
            }
        }
    });

    // Start the store handler in a thread
    let child = thread::spawn(move || loop {
        let action = rx.recv().unwrap();
        if backend == BACKEND_JSON {
            json_store::handle_action(action, path.clone());
        } else if backend == BACKEND_FILE {
            file_store::handle_action(action, path.clone());
        }
    });

    //Run infinitely, until CTRL+C
    let _res = child.join();
}
