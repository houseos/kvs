# kvs Makefile
# SPDX-License-Identifier: MIT
# Copyright (C) 2020 Benjamin Schilling

.PHONY: all build clean deps

all: clean deps build 

deps:
		curl https://sh.rustup.rs -sSf | sh -s -- -y && export PATH=$(PATH):$(HOME)/.cargo/bin && rustup toolchain install nightly && rustup default nightly && rustup component add rustfmt &&

build: deps
		export PATH=$(PATH):$(HOME)/.cargo/bin && cargo build --release	

clean:
		rm -f -r target
