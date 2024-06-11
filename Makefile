.PHONY: all build build-rust build-go

SHARED_LIB = ""
ifeq ($(OS),Windows_NT)
	SHARED_LIB = seda_tally_vm.dll
else
	UNAME_S := $(shell uname -s)
	ifeq ($(UNAME_S),Linux)
		SHARED_LIB = libseda_tally_vm.so
	endif
	ifeq ($(UNAME_S),Darwin)
		SHARED_LIB = libseda_tally_vm.dylib
	endif
endif

all: build test

build: build-rust #build-go

build-rust: build-rust-release

build-rust-debug:
	cargo build
	cp target/debug/$(SHARED_LIB) tallyvm/$(SHARED_LIB_DST)

build-rust-release:
	cargo build --release
	cp target/release/$(SHARED_LIB) tallyvm/$(SHARED_LIB_DST)

# build-go:
# 	(cd tallyvm && go build .)
