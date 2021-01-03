# kvs Makefile
# SPDX-License-Identifier: MIT
# Copyright (C) 2020 Benjamin Schilling

.PHONY: all build install clean uninstall

all: clean build

build:
		cargo build --release

clean:
		rm -f -r target
