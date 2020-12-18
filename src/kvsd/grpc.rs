/*
*  kvsd gRPC module
*  SPDX-License-Identifier: MIT
*  Copyright (C) 2020 Benjamin Schilling
*/

// Rust Standard Library
use std::net::SocketAddr;

// Two Lock Queue
use two_lock_queue::Sender;

// Tokio Imports for gRPC
use tokio::runtime::Runtime;
use tonic::{
    transport::{Identity, Server, ServerTlsConfig},
    Request, Response, Status,
};

// gRPC imports
use kvs_api::kvs_server::{Kvs, KvsServer};
use kvs_api::KeyValuePair;
pub mod kvs_api {
    tonic::include_proto!("kvs_api");
}

// kvs modules
use crate::store::file_store;
use crate::store::json_store;
use crate::store::store_actions::{QueueAction, ACTION_DELETE, ACTION_STORE};
use utils::{crypto, filesystem_wrapper::get_exec_dir, input_validation};

// Supported backends
const BACKEND_JSON: u8 = 0;
const BACKEND_FILE: u8 = 1;

// Implementation of the gRPC Service
//#[derive(Debug)]
pub struct KvsImpl {
    send_queue: Sender<QueueAction>,
    backend: u8,
    storage_path: String,
}

#[tonic::async_trait]
impl Kvs for KvsImpl {
    // storeValue Implementation
    async fn store(
        &self,
        request: Request<KeyValuePair>,
    ) -> Result<Response<KeyValuePair>, Status> {
        let message = request.into_inner();
        // sanitize key and value
        let key: String = message.key.trim().to_string();
        let value: String = message.value.trim().to_string();

        // Check key
        if !input_validation::validate_key(key.clone()) {
            return Err(Status::invalid_argument("Key invalid."));
        }
        // Dont check length if file backend is used.
        let mut check_length = true;
        if self.backend == BACKEND_FILE {
            check_length = false;
        }
        // Check value
        if !input_validation::validate_value(value.clone(), check_length) {
            return Err(Status::invalid_argument("Value invalid."));
        }
        // Check size of store if JSON Backend is used
        if self.backend == BACKEND_JSON && json_store::is_store_full() {
            return Err(Status::resource_exhausted(
                "Can not store more key value pairs, limit of 10.000 reached.",
            ));
        }
        // Create QueueAction and send it to queue
        let action: QueueAction = QueueAction {
            kv: KeyValuePair { key, value },
            action: ACTION_STORE,
        };
        self.send_queue.send(action).unwrap();

        Ok(Response::new(message))
    }
    // getValue Implementation
    async fn get(&self, request: Request<KeyValuePair>) -> Result<Response<KeyValuePair>, Status> {
        let message = request.into_inner();
        // sanitize key
        let key: String = message.key.trim().to_string();
        // Check key
        if !input_validation::validate_key(key.clone()) {
            return Err(Status::invalid_argument("Key invalid."));
        }
        let mut value: String = String::new();
        // If JSON Backend is used load from HashMap, otherwise load from file
        if self.backend == BACKEND_JSON {
            value = match json_store::get_value(key.clone()) {
                Ok(value) => value,
                Err(e) => return Err(Status::not_found(e)),
            };
        } else if self.backend == BACKEND_FILE {
            value = match file_store::get_value(key.clone(), self.storage_path.clone()) {
                Ok(value) => value,
                Err(e) => return Err(Status::not_found(e)),
            };
        }
        // Create response message
        let response_message: KeyValuePair = KeyValuePair { key, value };
        Ok(Response::new(response_message))
    }
    // deleteKey Implementation
    async fn delete(
        &self,
        request: Request<KeyValuePair>,
    ) -> Result<Response<KeyValuePair>, Status> {
        let message = request.into_inner();
        // sanitize key
        let key: String = message.key.trim().to_string();
        // Check key
        if !input_validation::validate_key(key.clone()) {
            return Err(Status::invalid_argument("Key invalid."));
        }

        if self.backend == BACKEND_JSON && !json_store::key_exists(key.clone()) {
            return Err(Status::not_found("Key not found!"));
        } else if self.backend == BACKEND_FILE && !file_store::key_exists(key.clone()) {
            return Err(Status::not_found("Key not found!"));
        }
        // Create QueueAction and send it to queue
        let action: QueueAction = QueueAction {
            kv: KeyValuePair {
                key: key.clone(),
                value: "".to_string(),
            },
            action: ACTION_DELETE,
        };
        self.send_queue.send(action).unwrap();

        // Create response message
        let response_message: KeyValuePair = KeyValuePair {
            key,
            value: "".to_string(),
        };
        Ok(Response::new(response_message))
    }
}

// Start the gRPC Server
pub fn start_grpc_server(
    ip: String,
    port: String,
    enable_tls: bool,
    send_queue: Sender<QueueAction>,
    backend: u8,
    storage_path: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let socket: SocketAddr = format!("{}:{}", ip, port).parse().unwrap();

    // If TLS is enabled start gRPC server with credentials
    if enable_tls {
        let path = get_exec_dir();
        println!(
            "TLS Option for gRPC given, looking for certificate and private key in {}",
            path
        );
        let credentials = match crypto::Credentials::new(path) {
            Ok(creds) => creds,
            Err(e) => return Err(e),
        };
        let identity =
            Identity::from_pem(credentials.get_certificate(), credentials.get_private_key());

        let kvs = KvsImpl {
            send_queue,
            backend,
            storage_path,
        };

        let mut rt = Runtime::new().expect("failed to obtain a new RunTime object");
        let server_future = Server::builder()
            .tls_config(ServerTlsConfig::new().identity(identity))?
            .add_service(KvsServer::new(kvs))
            .serve(socket);
        rt.block_on(server_future)
            .expect("failed to successfully run the future on RunTime");
    } else {
        let kvs = KvsImpl {
            send_queue,
            backend,
            storage_path,
        };
        println!("gRPC listening on {}", socket);
        let mut rt = Runtime::new().expect("failed to obtain a new RunTime object");
        let server_future = Server::builder()
            .add_service(KvsServer::new(kvs))
            .serve(socket);
        rt.block_on(server_future)
            .expect("failed to successfully run the future on RunTime");
    }
    Ok(())
}
