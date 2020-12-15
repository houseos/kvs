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
use crate::json_store::{get_value, QueueAction, ACTION_DELETE, ACTION_STORE};
use crate::utils::{crypto, filesystem_wrapper::get_exec_dir, input_validation};

// Implementation of the gRPC Service
//#[derive(Debug)]
pub struct KvsImpl {
    send_queue: Sender<QueueAction>,
}

#[tonic::async_trait]
impl Kvs for KvsImpl {
    // storeValue Implementation
    async fn store(
        &self,
        request: Request<KeyValuePair>,
    ) -> Result<Response<KeyValuePair>, Status> {
        let message = request.into_inner();

        if !input_validation::validate_key(message.clone().key) {
            return Err(Status::invalid_argument("Key invalid."));
        }
        if !input_validation::validate_value(message.clone().value) {
            return Err(Status::invalid_argument("Value invalid."));
        }
        let action: QueueAction = QueueAction {
            kv: message.clone(),
            action: ACTION_STORE,
        };
        self.send_queue.send(action).unwrap();

        Ok(Response::new(message))
    }
    // getValue Implementation
    async fn get(&self, request: Request<KeyValuePair>) -> Result<Response<KeyValuePair>, Status> {
        let message = request.into_inner();
        let value = match get_value(message.clone().key) {
            Ok(value) => value,
            Err(e) => return Err(Status::not_found(e)),
        };

        let response_message: KeyValuePair = KeyValuePair {
            key: message.clone().key,
            value: value,
        };
        Ok(Response::new(response_message))
    }
    // deleteKey Implementation
    async fn delete(
        &self,
        request: Request<KeyValuePair>,
    ) -> Result<Response<KeyValuePair>, Status> {
        let message = request.into_inner();
        let action: QueueAction = QueueAction {
            kv: message.clone(),
            action: ACTION_DELETE,
        };
        self.send_queue.send(action).unwrap();
        let response_message: KeyValuePair = KeyValuePair {
            key: message.clone().key,
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
) -> Result<(), Box<dyn std::error::Error>> {
    let socket: SocketAddr = format!("{}:{}", ip, port).parse().unwrap();

    // If TLS is enabled start gRPC server with credentials
    if enable_tls {
        let path = get_exec_dir().expect("Couldn't");
        println!(
            "TLS Option for gRPC given, looking for certificate and private key in {}",
            path.display()
        );
        let credentials = match crypto::Credentials::new(format!("{}", path.display())) {
            Ok(creds) => creds,
            Err(e) => return Err(e),
        };
        let identity =
            Identity::from_pem(credentials.get_certificate(), credentials.get_private_key());

        let kvs = KvsImpl {
            send_queue: send_queue,
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
            send_queue: send_queue,
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
