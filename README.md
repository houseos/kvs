# kvs - Secure Key Value Store

Have you ever implemented an application that had to store data persistently?
Have you ever been afraid of the confidentiality and integrity of this data?

Using **kvs** you can store your data protected by AES-256-GCM-SIV using hardware bound passwords.

The key-value-store-daemon **kvsd** takes care of storing your data and encrypting it at rest.
It provides a simple to use [gRPC](https://gRPC.io) interface that can be used in nearly any programming language.
If you want to run **kvsd** on a remote server you can enable TLS protection to ensure confidentialty during data transfer.

To be able to use the key-value store in shell scripts, **kvsc** provides a commandline interface for accessing the key-value-store.

## Commandline usage

### kvsd

```
kvsd 0.1.0
Benjamin Schilling <benjamin.schilling33@gmail.com>

USAGE:
    kvsd.exe [FLAGS] [OPTIONS]

FLAGS:
    -h, --help       Prints help information
        --tls        Set to enable TLS support for gRPC.
                     If set certificate and private key are expected as grpc.crt
                     and grpc.key in the execution directory of kvsd binary.    
    -V, --version    Prints version information

OPTIONS:
        --backend <backend>    Backend to be used. Default: "json"
                                [possible values: json, file]
        --ip <ip>              IP address the kvs daemon shall bind the gRPC interface to.
        --path <path>          Filesystem path for the persistent store.
        --port <port>          Port the kvs daemon shall bind the gRPC interface to.
```

### kvsc

```
kvsc 0.1.0
Benjamin Schilling <benjamin.schilling33@gmail.com>

USAGE:
    kvsc.exe [FLAGS] [OPTIONS] <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
        --tls        Set to enable TLS support for gRPC.
                     If set certificate and private key are expected as ca.crt in the execution directory of kvsc
                     binary.
    -V, --version    Prints version information

OPTIONS:
        --ip <ip>        IP address the kvs daemon is bound to.
        --port <port>    Port the kvs daemon is bound to.

SUBCOMMANDS:
    delete    Delete the given key.
    get       Get the value of a given key.
    help      Prints this message or the help of the given subcommand(s)
    store     Store a given key value pair.
```

### Options

#### TLS

**kvsd** & **kvsc** supports TLS protected gRPC connections. 
To use this feature you have to run the binaries with the `--tls` option.
Before doing this you should have generated a valid key pair using the following command and placed the private key (`grpc.key`) as well as certificate (`grpc.crt`) in the same directory as the **ksvd** binary.
The **ksvc** binary requires the CA certificate used to sign the ksvd certficiate (`ca.crt`) in the same directory as the **ksvc** binary.
To  protect the web filesystem a reverse proxy like nginx should be used.

**Beware: The following command generates an insecure self-signed certificate and should be used for development only!**

> `openssl req -newkey rsa:2048 -nodes -keyout grpc.key -x509 -days 365 -out grpc.crt`

Afterwards the generated certificate (**in a non-development environment the CA certificate**) can be added as the CA certificate in **ksvc** or any gRPC client, like BloomRPC, to establish TLS protected connections.

## Building the project

### Development

#### Build

`cargo build`

The gRPC code is generated during build using the `build.rs` build script.

#### Run

The following example uses only the mandatory parts of the arguments, for a full example see below.

`cargo run --bin kvsd`

`cargo run --bin kvsc`

### Release

#### Build

`cargo build --release`

The gRPC code is generated during build using the `build.rs` build script.

## Architecture

> **Hint:**
> Use the **`Markdown Preview Enhanced`** vscode plugin to render the PlantUML drawings.

The **kvsd** runs on a device. 

```plantuml
caption System Component Diagram

skinparam monochrome true
skinparam componentStyle uml2

scale max 650 width

component kvsd [
    <<component>>
    kvsd
]
component kvsc [
    <<component>>
    kvsc
]
component app [
    <<component>>
    Application
]

() ":27001\ngRPC" as gRPC


kvsd -- gRPC
kvsc -- gRPC
app -- gRPC

```

The **kvsd** consists of three threads.
The **main thread** is handling all commandline arguments, the **gRPC server thread** is receiving actions via gRPC and the **action queue thread** prevents concurrent writes (and deletes) to the store.

```plantuml
caption Sequence Diagram

skinparam monochrome true

participant ":client\n(e.g. kvsc)" as client

box "kvsd"
participant ":Main" as main
participant ":gRPC Server" as grpc
participant ":Store" as queue

activate main

main -> main: Parse command line arguments
main -> queue: Initialize
activate queue
queue -> queue: Parse persistent store depending on backend
queue --> main: Initialize
deactivate queue
main -> grpc: Start gRPC server
activate grpc
main -> queue: Start store action handler according to backend
activate queue

client -> grpc: Store(key, value)
activate grpc
grpc -> grpc: Validate input
grpc ->> queue: Dispatch store action
activate queue
grpc --> client: Store(key,value)
deactivate grpc
queue -> queue: Prepare encryption
queue -> queue: Encrypt and store
deactivate queue

client -> grpc: Get(key)
activate grpc
grpc -> grpc: Validate input
grpc -> grpc: Get value
grpc --> client: Get(key): Value
deactivate grpc

client -> grpc: Delete(key)
activate grpc
grpc -> grpc: Validate input
grpc ->> queue: Dispatch delete action
activate queue
grpc --> client: Delete(key)
deactivate grpc
queue -> queue: Delete key & value
deactivate queue

```


The **kvsd** is intended for embedded or industrial use cases where the hardware is limited but not too restricted. 
It can also be used in more powerful systems allowing more stored data.

## Security

This chapter describes the security goals of this project and how they are achieved.

**Goals:**

1. Data stored in the **kvsd** might be confidential, e.g. key material used by other processes or sensitive personal data.
2. Data stored in the **kvsd** must not be manipulated, e.g. configuration values of other services.

### Protecting data at rest

The data used by the **kvsd** is protected against manipulation and unallowed access.
For this purpose the data is encrypted using AES-256-GCM.
To not rely on a password that is supplied via commandline or stored in a hard-coded file,
password derivation is used.
For this purpose the **kvsd** integrates [*siemens/libuta*](https://github.com/siemens/libuta).
It uses it to derive hardware bound passwords from the stored derivation values.
This way the password used on each device running the **kvsd** are different, although the same derivation value might be used.

#### Protection in scope:

This mechanism should prevent **offline attacks**.

**Confidentiality:** The confidentiality is ensured because the encryption keys can only be derived on the hardware, otherwise brute-force is required.

**Integrity:** The integrity is ensured by using Authenticated Encryption, while the key required to generate a valid authentication tag can, analogous to the confidentiality, only be derived on the hardware.

#### Out of scope:

It does not protect against an attacker who can successfully boot the operating system or get access to the hardware trust anchor used by **siemens/libuta**.

### Protecting data in transit

When the **kvsd** is deployed in a different location than the client is run, data in transit should be protected.
Additionally the authenticity of the **kvsd** has to be ensured. 
For this purpose the **kvsd** can be supplied with credentials (private key & certificate) for TLS.

Additionally mutual authentication for TLS can be enabled to ensure only valid clients establish connections to the **kvsd**.

#### Protection in scope:

**Integrity:** During transit the requested data might not be manipulated.
E.g a manipulation of the values might lead to misbehaviour of a serivce retrieving its configuration this way.

**Confidentiality:** During transit the data has to be protected again unallowed access.
Otherwise an attacker might get access to sensitive information, e.g. the configuration of services that might be interesting for reconnaissance.

**Authenticity:** The client requesting data has to be sure that the received data originates from the **kvsd** and not a man in the middle.

**Authorization:** Only trustworthy clients may request data from the **kvsd**, therefore they have to provide a valid certificate on their own. Otherwise a malicous client could request arbitrary values from the **kvsd**, e.g. exploring the configuration of services that use **kvsd** as a storage.

#### Out of scope:

Restricting the access to data stored in **kvsd** to specific clients.
Since they have read & write access to all data, all clients have to be trustworthy.
If TLS is used, they have to provide a valid certificate to ensure this.

## Backends

The **kvsd** supports two backends.

The first backend is implemented as a JSON array of key value pairs.
It is inteded for little databases of keys with short values because the whole store is parsed and stored in RAM.

The second backend is implemented on file base.
Each entry is stored as a separate file with the content representing the value.
The keys are mapped to random file names using an encrypted meta-data file.

### JSON Backend

The JSON Backend stores all values in the following structure:

```json
{
    "key": "value",
    "key2": "value2",
    "key n": "value n"
}
```

On start-up of the **kvsd** the file is parsed and stored in a hashmap.
Therefore all data is stored in RAM.
Using this backend the length of the values is restricted to 1024 characters.
Otherwise the risk of consuming to much RAM during runtime is too high.
Additionally the number of key value pairs is restricted to 10.000 to prevent too much ressource consumption.
10.000 key value pairs should result in around 11MB of RAM consumption.

#### Security

The JSON Backend is intended for less-secure environments.
The keys are stored in plain-text form, while only the values are encrypted.

The **kvsd** stores the data it receives in a JSON file.
This JSON file contains key value pairs.

```json
{
  "key": "derivation-value (32 byte)$iv (16 bytes)$encrypted-value",
  "key": "01234567890123450123456789012345$0123456789012345$0123456789012345$",
  "test": "testvalue"
}
```
The encrypted value is generated using the function `AES-256-GCM-SIV(Key, Initialization Vector, Plaintext)` and Base64 encoded as follows: 

> `Base64(AES-256-GCM-SIV(derive_key(dv), iv, plaintext value))`

### File Backend

The file backend stores the value of each key value pair in a separate file.
Additionally a JSON file is created containing the key, the references to all value files, their derivation values and IVs required for the decryption.

This allows to store dramatically bigger values since the values of all keys are not stored in RAM during runtime.
Only the JSON file is loaded during runtime and a file containing a requested value is loaded and decrypted only on request.

#### Security

The values are stored in files with random file names.
Their purpose can be retrieved from the JSON file containing the meta data.
To prevent reading the meta-data while the **kvsd** is not running, the file is encrypted.

The key is of each file is derived using a random derivation value.
Additionally a random IV is generated for each file.
These two measures should prevent IV reuse for the same key although AES-256-GCM-SIV is used.

```json
{
    "<key 1>": {
        "filename": "<random filename>",
        "derivation_value": "<random key derivation value>",
        "initialization_vector": "<random IV>"
    },
    "<key 2>": {
        "filename": "<random filename>",
        "derivation_value": "<random key derivation value>",
        "initialization_vector": "<random IV>"
    },
    "<key n>": {
        "filename": "<random filename>",
        "derivation_value": "<random key derivation value>",
        "initialization_vector": "<random IV>"
    }
}
```

The encrypted content is created in the same way as the value is encrypted in the JSON backend.

## License

SPDX-License-Identifier: MIT

## Copyright 

Copyright (C) 2020 Benjamin Schilling