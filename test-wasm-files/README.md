# Test WASM Files

This directory contains various WASM files (and sometimes the wat files) used in the tests.

## WASM Files

### assign_too_much_memory.wasm

Source: ???
Used to check that when a WASM binary attempts to allocate a large amount of memory it runs out of gas before impacting the host.

### cache_misses.wasm

Source: `cache_misses.wat`.
Used to check that metering is injected before branch sources, not only at the end of a branch.

### integration_test.wasm

Source: https://github.com/sedaprotocol/seda-sdk/tree/main/libs/as-sdk-integration-tests
Used to test the VM in general as well as specific host function imports.

### null_byte_string.wasm

Source: ???
Used to verify that a null byte in the output does not panic at the VM or FFI layers.

### price-feed-playground.wasm

Source: Internal DR repo.
Used to verify that a common usecase works as expected when built with the Rust SDK, as well as the amount of gas consumed.

### randomNumber.wasm

Source: Internal DR repo.
Used to verify that an import which is available in the DR VM (randomness in this case) does not crash the VM as long as it's not called.

### simplePriceFeed.wasm

Source: Internal DR repo
Used to verify that a common usecase works as expected when built with the AssemblyScript SDK, as well as the amount of gas consumed.

### spam-fd-write.wasm

Source `spam-fd-write.wat`
Used to verify that calling `fd-write` directly does not have a significant impact on the compute time.

### stdout_null_bytes.wasm

Source: ???
Used to verify that a null byte in stdout does not panic at the VM or FFI layers.

### tally.wasm

Source: ???
Used to test some common scenarios.

### test-vm.wasm

Source: `./test-vm`
Used to test various edge cases.
