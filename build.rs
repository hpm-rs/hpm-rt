use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Put the linker script somewhere the linker can find it
    if cfg!(feature = "flash-xip") {
        fs::write(out_dir.join("link.x"), include_bytes!("flash_xip.x")).unwrap();
    } else {
        fs::write(out_dir.join("link.x"), include_bytes!("ram.x")).unwrap();
    }
    println!("cargo:rustc-link-search={}", out_dir.display());
    println!("cargo:rerun-if-changed=link.x");
}
