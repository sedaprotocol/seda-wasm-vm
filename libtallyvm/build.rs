use std::env;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    println!("cargo:warning={:?}", &crate_dir);

    cbindgen::generate(crate_dir)
        .expect("Unable to generate bindings")
        .write_to_file("../tallyvm/libseda_tally_vm.h");

    // Set the linker flags for static linking with musl
    // if env::var("TARGET").unwrap().contains("musl") {
    //     println!("cargo:rustc-link-lib=static=m");
    //     println!("cargo:rustc-link-lib=static=c");
    //     println!("cargo:rustc-link-search=/usr/aarch64-linux-gnu/lib");
    //     println!("cargo:rustc-link-search=/usr/lib/x86_64-linux-musl");
    // }
}
