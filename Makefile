# kvs
# Key Value Store
# SPDX-License-Identifier: MIT
# Copyright (C) 2020 Benjamin Schilling

.PHONY: all build install clean uninstall

all: clean build

build:
		cargo build --release

install: 
		mkdir -p $(DESTDIR)/usr/local/bin/
		cp -r target/release/kvsd  $(DESTDIR)/usr/local/bin/kvsd
                cp -r target/release/kvsc  $(DESTDIR)/usr/local/bin/kvsc

clean:
		rm -f -r target

uninstall:
		rm -r $(DESTDIR)/usr/local/bin/kvsd
                rm -r $(DESTDIR)/usr/local/bin/kvsc
