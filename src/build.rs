use std::{env, fs, path::PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let exe_dir = out_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    fs::copy("src/WinRing0x64.dll", exe_dir.join("WinRing0x64.dll")).expect("Failed to copy DLL");
}
