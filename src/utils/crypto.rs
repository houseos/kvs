/*
*  crypto utils module
*  SPDX-License-Identifier: MIT
*  Copyright (C) 2020 Benjamin Schilling
*/

#[derive(Debug)]
pub struct Credentials {
    private_key: String,
    certificate: String,
}

impl Credentials {
    pub fn new(path: String) -> Result<Credentials, Box<dyn std::error::Error>> {
        let cert = match std::fs::read_to_string(format!("{}/grpc.crt", path)) {
            Ok(c) => c,
            Err(e) => return Err(Box::new(e)),
        };
        let key = match std::fs::read_to_string(format!("{}/grpc.key", path)) {
            Ok(k) => k,
            Err(e) => return Err(Box::new(e)),
        };
        Ok(Credentials {
            private_key: key,
            certificate: cert,
        })
    }

    pub fn get_private_key(&self) -> &[u8] {
        self.private_key.as_bytes()
    }

    pub fn get_certificate(&self) -> &[u8] {
        self.certificate.as_bytes()
    }
}

#[derive(Debug)]
pub struct TrustStore {
    ca_cert: String,
}

impl TrustStore {
    pub fn new(path: String) -> Result<TrustStore, Box<dyn std::error::Error>> {
        let ca_cert = match std::fs::read_to_string(format!("{}/ca.crt", path)) {
            Ok(c) => c,
            Err(e) => return Err(Box::new(e)),
        };
        Ok(TrustStore { ca_cert: ca_cert })
    }

    pub fn get_trusted_certificate(&self) -> &[u8] {
        self.ca_cert.as_bytes()
    }
}
