use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::fs;

// Copy the config file from /src to the output directory, next to our resulting executable
pub fn main() {
    let cargo_manifest_dir = env::var_os("CARGO_MANIFEST_DIR").unwrap_or(OsString::from("no_manifest_dir"));
    let output_dir = env::var_os("OUT_DIR").unwrap_or(OsString::from("no_output_dir"));
    let exe_dir = Path::new(output_dir.to_str().unwrap())
                      .parent().unwrap()
                      .parent().unwrap()
                      .parent().unwrap();
    
    let mut config_file = PathBuf::new();
    config_file.push(&cargo_manifest_dir);
    config_file.push("src");
    config_file.push("config.toml");

    let dest_file = Path::new(exe_dir).join("config.toml");
    match fs::copy(&config_file, &dest_file){
        Ok(_) => println!("Copied config file successfully from {:?} to {:?}", config_file, dest_file),
        Err(e) => println!("Failed to copy script from {:?} to {:?}: {:?}",config_file, dest_file, e)
    }
}