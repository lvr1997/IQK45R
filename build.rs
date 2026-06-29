// build.rs
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let vial_path = PathBuf::from("vial.json");

    if vial_path.exists() {
        fs::copy(&vial_path, out_dir.join("vial.json")).unwrap();
        println!("cargo:rerun-if-changed=vial.json");
    }
}