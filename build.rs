/*
*  build script for gRPC code generation
*  SPDX-License-Identifier: MIT
*  Copyright (C) 2020 Benjamin Schilling
*/

use std::path::Path;

fn main() -> Result<(), String> {
    let proto_file = "proto/kvs.proto";
    // Check that proto file exists
    if !Path::new(proto_file).exists() {
        return Err("Proto file does not exist.".to_string());
    }
    // Generate code from proto file
    match tonic_build::compile_protos(proto_file) {
        Ok(_o) => Ok(()),
        Err(e) => Err(format!("Failed: {:?}", e)),
    }
}
