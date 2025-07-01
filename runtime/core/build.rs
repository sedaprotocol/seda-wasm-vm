// build.rs
use cargo_metadata::MetadataCommand;

const PACKAGE_NAMES: [&str; 4] = ["wasmer", "wasmer-types", "wasmer-middlewares", "wasmer-wasix"];

fn main() {
    let meta = MetadataCommand::new().exec().unwrap();
    let versions: Vec<String> = PACKAGE_NAMES
        .iter()
        .filter_map(|name| {
            meta.packages
                .iter()
                .find(|p| p.name.as_ref() == *name)
                .map(|p| p.version.to_string())
        })
        .collect();

    let wasmer_version = versions.first().unwrap();
    let wasmer_types_version = versions.get(1).unwrap();
    let wasmer_middlewares_version = versions.get(2).unwrap();
    let wasmer_wasix_version = versions.get(3).unwrap();

    println!("cargo:rustc-env=WASMER_VERSION={}", wasmer_version);
    println!("cargo:rustc-env=WASMER_TYPES_VERSION={}", wasmer_types_version);
    println!(
        "cargo:rustc-env=WASMER_MIDDLEWARES_VERSION={}",
        wasmer_middlewares_version
    );
    println!("cargo:rustc-env=WASMER_WASIX_VERSION={}", wasmer_wasix_version);

    // Print the versions to the console for debugging purposes
    println!("wasmer version: {}", wasmer_version);
    println!("wasmer types version: {}", wasmer_types_version);
    println!("wasmer middlewares version: {}", wasmer_middlewares_version);
    println!("wasmer wasix version: {}", wasmer_wasix_version);
}
