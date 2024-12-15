use std::env;
use std::path::Path;

fn main() {
    // Tell Cargo that if the database schema changes, rerun the build script
    println!("cargo:rerun-if-changed=migrations/*");
}