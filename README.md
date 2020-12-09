# kvs - Non-distributed Secure Key Value Store

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

## License

SPDX-License-Identifier: MIT

## Copyright 

Copyright (C) 2020 Benjamin Schilling