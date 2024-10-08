use std::env;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    println!("cargo:warning={:?}", &crate_dir);

    cbindgen::generate(crate_dir)
        .expect("Unable to generate bindings")
        .write_to_file("../tallyvm/libseda_tally_vm.h");
}
