use std::env::var;
use std::error::Error;
use std::fs::copy;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let cwd = PathBuf::from(var("CARGO_MANIFEST_DIR").unwrap());
    let root = cwd.parent().unwrap().parent().unwrap();
    let license = root.join("LICENSE");

    if license.exists() {
        copy(root.join("LICENSE"), cwd.join("LICENSE"))?;
    }

    Ok(())
}