.PHONY: all build build-rust

SHARED_LIB = ""
ifeq ($(OS),Windows_NT)
	SHARED_LIB = wasmvm.dll
else
	UNAME_S := $(shell uname -s)
	ifeq ($(UNAME_S),Linux)
		SHARED_LIB = libwasmvm.so
	endif
	ifeq ($(UNAME_S),Darwin)
		SHARED_LIB = libwasmvm.dylib
	endif
endif

USER_ID := $(shell id -u)
USER_GROUP = $(shell id -g)

# TODO Update libwasmvm.h?

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
##                        Building Shared Libraries                          ##
###############################################################################
docker-image-centos7:
	docker build --pull . --platform linux/x86_64 -t seda-wasm-vm-builder-centos7 -f ./Dockerfile.centos7

# For building glibc Linux shared libraries (.so)
release-build-centos7:
	rm -rf target/x86_64-unknown-linux-gnu/release
	rm -rf target/aarch64-unknown-linux-gnu/release
	docker run --rm -u $(USER_ID):$(USER_GROUP) -v $(shell pwd):/code seda-wasm-vm-builder-centos7 build_linux.sh
	cp target/x86_64-unknown-linux-gnu/release/libwasmvm.so tallyvm/libwasmvm.x86_64.so
	cp target/aarch64-unknown-linux-gnu/release/libwasmvm.so tallyvm/libwasmvm.aarch64.so


###############################################################################
##                        Building Static Libraries                          ##
###############################################################################
docker-image-alpine:
	docker build . -t seda-wasm-vm-builder-alpine

# For building musl Linux static libraries (.a)
release-build-alpine:
	rm -rf target/aarch64-unknown-linux-musl/release
	rm -rf target/x86_64-unknown-linux-musl/release
	docker run --rm -u $(USER_ID):$(USER_GROUP) -v $(shell pwd):/code seda-wasm-vm-builder-alpine
	cp target/aarch64-unknown-linux-musl/release/libwasmvm.a tallyvm/libwasmvm.aarch64.a
	cp target/x86_64-unknown-linux-musl/release/libwasmvm.a tallyvm/libwasmvm.x86_64.a

.PHONY: docker-image-alpine release-build-alpine
