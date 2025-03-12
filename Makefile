.PHONY: all build build-rust build-test

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

fmt:
	cargo +nightly fmt --all

check:
	cargo clippy --all-features --locked -- -D warnings

# TODO Update libseda_tally_vm.h?
all: build test

build: build-rust

build-rust: build-rust-release

build-test: 
	cargo build -p test-vm --target wasm32-wasip1;

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
	cp target/x86_64-unknown-linux-gnu/release/libseda_tally_vm.so tallyvm/libseda_tally_vm.x86_64.so
	cp target/aarch64-unknown-linux-gnu/release/libseda_tally_vm.so tallyvm/libseda_tally_vm.aarch64.so


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
	cp target/aarch64-unknown-linux-musl/release/libseda_tally_vm.a tallyvm/libseda_tally_vm.aarch64.a
	cp target/x86_64-unknown-linux-musl/release/libseda_tally_vm.a tallyvm/libseda_tally_vm.x86_64.a

.PHONY: docker-image-alpine release-build-alpine
