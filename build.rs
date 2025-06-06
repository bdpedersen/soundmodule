use std::env;
use std::path::PathBuf;

fn main() {
    let header_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("include")
        .join("soundmodule.h");

    println!("cargo::rustc-env=DEP_SOUNDMODULE_INCLUDE={}", header_path.parent().unwrap().display());
}