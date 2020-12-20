/*
*  kvsc main
*  SPDX-License-Identifier: MIT
*  Copyright (C) 2020 Benjamin Schilling
*/

#![feature(proc_macro_hygiene, decl_macro)]

// Rust Standard Library
use std::io::{self, Read};

//tonic
use tonic::transport::{Certificate, ClientTlsConfig};

// gRPC imports
use kvs_api::kvs_client::KvsClient;
use kvs_api::KeyValuePair;
pub mod kvs_api {
    tonic::include_proto!("kvs_api");
}

//kvs crates
use utils::{crypto, filesystem_wrapper::get_exec_dir, input_validation};

// CLI interface
extern crate clap;
use clap::{App, AppSettings, Arg, SubCommand};

const INPUT_CLI: u8 = 0;
const INPUT_PIPE: u8 = 1;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Specify commandline arguments
    let matches = App::new("kvsc")
        .version(clap::crate_version!())
        .author("Benjamin Schilling <benjamin.schilling33@gmail.com>")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg(
            Arg::with_name("ip")
                .help("IP address the kvs daemon is bound to.")
                .required(false)
                .long("ip")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("port")
                .help("Port the kvs daemon is bound to.")
                .required(false)
                .long("port")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("tls")
                .help("Set to enable TLS support for gRPC.\nIf set certificate and private key are expected as ca.crt in the execution directory of kvsc binary.")
                .long("tls"),
        )
        .subcommand(
            SubCommand::with_name("store")
            .about("Store a given key value pair.")
            .arg(
                Arg::with_name("key")
                .long("key")
                .help("Key of the key value pair, max. length 32.")
                .takes_value(true)
                .required(true)
            )
            .arg(
                Arg::with_name("value")
                .long("value")
                .help("Value of the key value pair, max. length 1024.")
                .takes_value(true)
            )
            .arg(
                Arg::with_name("pipe")
                .long("pipe")
                .help("Allows piping the value in kvsc, alternative for \"value\".\n(No size restriction with file backend, \nbest used with base64 strings or complete files.)")
            )
        )
        .subcommand(
            SubCommand::with_name("get")
            .about("Get the value of a given key.")
            .arg(
                Arg::with_name("key")
                .long("key")
                .help("Key of the key value pair, max. length 32.")
                .takes_value(true)
                .required(true)
            )
        )
        .subcommand(
            SubCommand::with_name("delete")
            .about("Delete the given key.")
            .arg(
                Arg::with_name("key")
                .long("key")
                .help("Key of the key value pair, max. length 32.")
                .takes_value(true)
                .required(true)
            )
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

    // create a channel for the connection to the server
    let socket = format!("http://{}:{}", ip, port).parse().unwrap();
    let channel;
    if matches.is_present("tls") {
        let path = get_exec_dir();
        println!("TLS Option for gRPC given, looking for ca.crt in {}", path);
        let trust_store = match crypto::TrustStore::new(path) {
            Ok(trusted) => trusted,
            Err(e) => {
                eprintln!("Error during store: {:?}", e);
                std::process::exit(0x0001);
            }
        };
        let cert = Certificate::from_pem(trust_store.get_trusted_certificate());

        channel = tonic::transport::Channel::builder(socket)
            .tls_config(ClientTlsConfig::new().ca_certificate(cert))?
            .connect()
            .await?;
    } else {
        channel = tonic::transport::Channel::builder(socket).connect().await?;
    }

    // create a gRPC client from the channel
    let mut client = KvsClient::new(channel);

    // handle subcommands
    match matches.subcommand() {
        ("store", Some(sub_m)) => {
            // Perform input validation on options
            if !input_validation::validate_key(sub_m.value_of("key").unwrap().to_string()) {
                eprintln!("Provided key invalid.");
                std::process::exit(0x0001);
            }
            // Check whether either value or pipe are given
            let mut _value_input: u8 = INPUT_CLI;
            if sub_m.is_present("value") && !sub_m.is_present("pipe") {
                _value_input = INPUT_CLI;
            } else if !sub_m.is_present("value") && sub_m.is_present("pipe") {
                _value_input = INPUT_PIPE;
            } else {
                // If both value and pipe are given exit.
                eprintln!("Provide either \"value\" or \"pipe\".");
                std::process::exit(0x0001);
            }
            // for value perform input validation
            if _value_input == INPUT_CLI
                && !input_validation::validate_value(
                    sub_m.value_of("value").unwrap().to_string(),
                    false,
                )
            {
                eprintln!("Provided value invalid.");
                std::process::exit(0x0001);
            }

            // Get values of arguments
            let key = sub_m.value_of("key").unwrap().to_string();
            let mut value = String::new();
            if _value_input == INPUT_CLI {
                value = sub_m.value_of("value").unwrap().to_string();
            } else {
                match io::stdin().read_to_string(&mut value) {
                    Ok(string) => string,
                    Err(e) => {
                        eprintln!("Could not read value from stdin: {}.", e);
                        std::process::exit(0x0001);
                    }
                };
            }
            // creating a new Request
            let request = tonic::Request::new(KeyValuePair { key, value });
            // Send request and handle response
            match client.store(request).await {
                Ok(response) => {
                    println!("Storing key \"{}\" successful.", response.into_inner().key);
                    std::process::exit(0x0000);
                }
                Err(e) => {
                    eprintln!("Error during store: {:?}", e.message());
                    std::process::exit(0x0001);
                }
            };
        }
        ("get", Some(sub_m)) => {
            // Perform input validation on options
            if !input_validation::validate_key(sub_m.value_of("key").unwrap().to_string()) {
                eprintln!("Provided key invalid.");
                std::process::exit(0x0001);
            }
            // Get values of options
            let key = sub_m.value_of("key").unwrap().to_string();
            // creating a new Request
            let request = tonic::Request::new(KeyValuePair {
                key,
                value: "".to_string(),
            });
            // Send request and handle response
            match client.get(request).await {
                Ok(response) => {
                    println!("{}", response.into_inner().value);
                    std::process::exit(0x0000);
                }
                Err(e) => {
                    eprintln!("Error during get: {:?}", e.message());
                    std::process::exit(0x0001);
                }
            };
        }
        ("delete", Some(sub_m)) => {
            // Perform input validation on options
            if !input_validation::validate_key(sub_m.value_of("key").unwrap().to_string()) {
                eprintln!("Provided key or value invalid.");
                std::process::exit(0x0001);
            }
            // Get values of options
            let key = sub_m.value_of("key").unwrap().to_string();
            // creating a new Request
            let request = tonic::Request::new(KeyValuePair {
                key,
                value: "".to_string(),
            });

            // Send request and handle response
            match client.delete(request).await {
                Ok(response) => {
                    println!("Deleting key \"{}\" successful.", response.into_inner().key);
                    std::process::exit(0x0000);
                }
                Err(e) => {
                    eprintln!("Error during delete: {:?}", e.message());
                    std::process::exit(0x0001);
                }
            };
        }
        _ => {
            eprintln!("Unknown subcommand.");
            std::process::exit(0x0001);
        }
    };
}
