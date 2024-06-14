.PHONY: all build build-rust

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

USER_ID := $(shell id -u)
USER_GROUP = $(shell id -g)

# TODO Update libseda_tally_vm.h?

all: build test

build: build-rust

build-rust: build-rust-release

build-rust-debug:
	cargo build
	cp target/debug/$(SHARED_LIB) tallyvm/$(SHARED_LIB_DST)

build-rust-release:
	cargo build --release
	cp target/release/$(SHARED_LIB) tallyvm/$(SHARED_LIB_DST)


###############################################################################
##                             Static Library                                ##
###############################################################################
docker-image-alpine:
	docker build . -t seda-wasm-vm-builder-alpine

# For building the static library for Alpine Linux (.a)
release-build-alpine:
	rm -rf target/aarch64-unknown-linux-musl/release
	rm -rf target/x86_64-unknown-linux-musl/release
	docker run --rm -u $(USER_ID):$(USER_GROUP) -v $(shell pwd):/code seda-wasm-vm-builder-alpine
	cp target/aarch64-unknown-linux-musl/release/libseda_tally_vm.a tallyvm/libseda_tally_vm.aarch64.a
	cp target/x86_64-unknown-linux-musl/release/libseda_tally_vm.a tallyvm/libseda_tally_vm.x86_64.a

.PHONY: docker-image-alpine release-build-alpine
