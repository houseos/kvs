# kvs Makefile
# SPDX-License-Identifier: MIT
# Copyright (C) 2020 Benjamin Schilling

.PHONY: all build install clean uninstall

all: clean build

build:
		curl https://sh.rustup.rs -sSf | sh -s -- -y
		export PATH=$PATH:$HOME/.cargo/bin
		rustup toolchain install nightly
		rustup default nightly
		export PROTOC=/usr/bin/protoc && export PROTOC_INCLUDE=/usr/include && env && cargo build --release	

clean:
		rm -f -r target
