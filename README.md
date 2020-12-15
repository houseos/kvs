# kvs - Secure Key Value Store

Have you ever implemented an application that had to store data persistently?
Have you ever been afraid of the confidentiality of this information?

Using **kvs** you can store your data protected by AES-256-GCM using hardware bound passwords.

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
                     and grpc.key in the execution directory of kvsc binary.
    -V, --version    Prints version information

OPTIONS:
        --ip <ip>        IP address the kvs daemon shall bind the gRPC interface to.
        --port <port>    Port the kvs daemon shall bind the gRPC interface to.
```

### kvsc

```
kvsc 0.1.0
Benjamin Schilling <benjamin.schilling33@gmail.com>

USAGE:
    kvsc.exe [FLAGS] [OPTIONS] [SUBCOMMAND]

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

kvsd & kvsc supports TLS protected gRPC connections. 
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

```puml
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

The kvsd is intended for embedded or industrial use cases where the hardware is limited but not too restricted. 
It can also be used in more powerful systems allowing more stored data.



## Backends

The kvsd supports two backends.

The first backend is implemented as a JSON array of key value pairs.
It is inteded for small databases of keys with small keys because the whole store is parsed and stored in RAM.

The second backend is implemented on file base.
Each entry is stored as a separate file with the file name representing the key and the content representing the value.

### JSON Backend

The JSON Backend stores all values in the following structure:

```json
{
    "key": "value",
    "key2": "value2",
    "key n": "value n"
}
```

On start-up of the kvsd the file is parsed and stored in a hashmap.
Therefore all data is stored in RAM.
Using this backend the length of the values is restricted to 1024 characters.
Otherwise the risk of consuming to much RAM during runtime is too high.
Additionally the number of key value pairs is restricted to 10.000 to prevent too much ressource consumption.
10.000 key value pairs result in around 11MB of RAM consumption.

#### Security

The JSON Backend is intended for less-secure environments.
The keys are stored in plain-text form, while only the values are encrypted.



The kvsd stores the data it receives in a JSON file.
This JSON file contains key value pairs.

```json
{
  "key": "derivation-value (32 byte)$iv (16 bytes)$encrypted-value",
  "key": "01234567890123450123456789012345$0123456789012345$0123456789012345$",
  "test": "testvalue"
}
```
The encrypted value is generated using the function `Base64(AES256GCM(Key, Initialization Vector, Plaintext))` as follows: 

> `Base64(AES256GCM(derive_key(dv), iv, plaintext value))`

```puml
```

### File Backend

The file backend stores each key value pair in a file.
Additionally a JSON file is created containing references to all files and the key related to each of them.

This allows to store dramatically bigger values.
Only the JSON file is loaded during runtime and the file is loaded, decrypted and returned only on request.

#### Security

The key is stored in an encrypted JSON file, containing a reference to an encrypted file (containing the value) and the secrect to decrypt the file.
The JSON file is decrypted with a secret derived from the 

```json
{
    "files": [
        {
            "key": "key",
            "filename": "file1",
            "secret": "key"
        },
        {
            "key": "key",
            "filename": "file2",
            "secret": "key"
        }
    ]
}
```

## License

SPDX-License-Identifier: MIT

## Copyright 

Copyright (C) 2020 Benjamin Schilling