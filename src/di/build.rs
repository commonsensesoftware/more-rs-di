use std::env::var;
use std::error::Error;
use std::fs::copy;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let cwd = PathBuf::from(var("CARGO_MANIFEST_DIR").unwrap());
    let root = cwd.parent().unwrap().parent().unwrap();
    let src = root.join("README.md");
    let dst = cwd.join("README.md");

    println!("cargo:rerun-if-changed={}", src.display());

    if dst.exists() {
        println!("cargo:rerun-if-changed={}", dst.display());
    } else {
        copy(src, dst)?;
    }

    Ok(())
}
