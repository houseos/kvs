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
mod json_store;
use crate::utils::input_validation;
use json_store::QueueAction;
use utils;

// CLI interface
extern crate clap;
use clap::{App, Arg};

// CLI Signal handling
extern crate ctrlc;

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
            Arg::with_name("tls")
                .help("Set to enable TLS support for gRPC.\nIf set certificate and private key are expected as grpc.crt\nand grpc.key in the execution directory of kvsc binary.")
                .long("tls"),
        )
        .get_matches();
    // Set IP and Port to default values
    let mut ip: String = "127.0.0.1".to_string();
    let mut port: String = "27001".to_string();
    // Set IP to provided value if existing
    if matches.is_present("ip") {
        if input_validation::validate_ipv4(matches.value_of("ip").unwrap().to_string()) {
            ip = matches.value_of("ip").unwrap().to_string();
        } else {
            eprintln!(
                "IP parameter {} invalid, only IPv4 allowed.",
                matches.value_of("ip").unwrap().to_string()
            );
            std::process::exit(0x0001);
        }
    }
    // Set Port to provided value if existing
    if matches.is_present("port") {
        if input_validation::validate_port(matches.value_of("port").unwrap().to_string()) {
            port = matches.value_of("port").unwrap().to_string();
        } else {
            eprintln!(
                "Port parameter {} invalid, only valid TCP port numbers allowed.",
                matches.value_of("port").unwrap().to_string()
            );
            std::process::exit(0x0001);
        }
    }

    // Read persistent store from file
    match json_store::initialize_store_from_file() {
        Ok(ok) => println!("Finished loading file: {}", ok),
        Err(e) => eprintln!("Error loading file: {}", e),
    }

    let (tx, rx) = two_lock_queue::unbounded::<QueueAction>();

    // Start the gRPC Server in a thread
    thread::spawn(move || {
        match grpc::start_grpc_server(ip, port, matches.is_present("tls"), tx.clone()) {
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
        json_store::handle_action(action);
    });

    //Run infinitely, until CTRL+C
    let _res = child.join();
}
