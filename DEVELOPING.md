# Developing

[1]: https://rustup.rs/
[2]: https://go.dev/
[3]: https://nexte.st/

## Environment set up

- Install [rustup][1]. Once installed, make sure you have the wasm32 target:

  ```bash
  rustup default stable
	# For formatting the rust code nightly is needed
	rustup install nightly
  ```

- Install [go][2]: From their downloads page or your disto's package manager(brew,win-get,apt,etc).

## Code Layout

- `runtime`: Contains two rust libraries:
  - `core`: The core VM library made using `wasmer`, and `wasmer-wasix` to define rust imports used in the VM.
  - `sdk`: Various functionalities used to help write `rust` WASMs for the VM.
- `libtallyvm`: A rust library that leverages the [cbindgen](https://crates.io/crates/cbindgen) crate to help write a `c` header file` and compile the code to `c` library files.
- `tallyvm`: The Go library that uses CGO directives to bind to the libraries.

## Compiling

Compiling is complex due to us needing libraries for the different architectures.

You can compile and test on your own architecture, as long as you aren't on windows.

So as a work around we have a manual CI that can be triggered to upload the library files(necessary for the go side). This does *NOT* include macos, please ask a friendly macos dev to help you out there.

### Updating The Library Files Via The Manual CI
1. Go to the `Actions` tab on Github.
2. Click `Manual Build` on the left hand side bar.
3. Click `Run workflow`.
	1. Select the branch.
	2. Enter the CI password(note what you type is in plain text).
	3. Make sure `all` is selected.
	4. Do not click `Enable debug mode`.
4. A build will show up in the `workflow runs` section.
5. Once that has a green check mark next to it you can click on the `Manual Build` next to it.
6. At the bottom of that page it shows `Artifacts`.
	1. If you followed step 3 it should be named `branch-refs-heads-feat-proxy_http_fetch-import-arch-all-debug-false`.
7. Click the download icon next to the artifact name and it will download as a zip file.
8. Place those files in the `tallyvm` directory overwriting the files.
	1. Note this does not update the macOS one. To build that one on a macOS machine run `cargo build --release`, and copy the produced `.dylib` file to the `tallyvm` directory.

## Linting

`rustfmt` is used to format any Rust source code, we do use nightly format features: `cargo +nightly fmt`.

Nightly can be installed with: `rustup install nightly`. 

`clippy` is used as the linting tool: `cargo clippy -- -D warnings`

## Testing

### Rust Unit

Rust unit testing can be done with: `cargo test`.

You could also install [nextest][4], with `cargo install cargo-nextest --locked`, then run `cargo nextest run --workspace`. Nextest is a faster test runner for Rust.

### Go Unit

Go unit testing can be done with `go test` if you are in the `tallyvm` directory.

### Test Wasms

The file `./integration-test.wasm` is taken from the SEDA-SDK integration tests: https://github.com/sedaprotocol/seda-sdk/tree/main/libs/as-sdk-integration-tests.

While the others are from an internal closed repo.

## xtask

We use `cargo xtask` to help automate lots of various actions.
It doesn't require any additional installations to use `xtask`, its just a more rust-esque way of doing a `Makefile`.

You can read more about [xtask](https://github.com/matklad/cargo-xtask) and it's benefits at that link.

It currently offers commands:
	- `compile`: To help cross compile the libraries. Does require you to have all the tools setup to do that.
	- `apt-install`: To help install cross compilation tools on distros using the `apt` package manager.
	- `cov`: To run test coverage locally. For this one, it's the column with the header `Cover` that determines overall coverage percentage.
	- `cov-ci`: To run test coverage in CI.
