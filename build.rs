use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::fs;

// Copy the config file and log config file from /config to the output directory, next to our resulting executable
pub fn main() {
    let cargo_manifest_dir = env::var_os("CARGO_MANIFEST_DIR").unwrap_or_else(|| {
        panic!("BUILD: Unable to determine CARGO_MANIFEST_DIR! Aborting build process...");
    });
    let output_dir = env::var_os("OUT_DIR").unwrap_or_else(|| {
        panic!("BUILD: Unable to retrieve OUT_DIR environment varraible. Error: {}. Aborting build proces...");
    });
    let exe_dir = Path::new(output_dir.to_str().unwrap())
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    let mut config_file = PathBuf::new();
    config_file.push(&cargo_manifest_dir);
    config_file.push("config");
    config_file.push("config.toml");

    let dest_file = Path::new(exe_dir).join("config.toml");

    println!(
        "BUILD: Attempting to move {:?} to {:?}",
        config_file, dest_file
    );

    match fs::copy(&config_file, &dest_file) {
        Ok(_) => println!(
            "BUILD: Copied config file successfully from {:?} to {:?}",
            config_file, dest_file
        ),
        Err(e) => println!(
            "BUILD: Failed to copy script from {:?} to {:?}: {:?}.",
            config_file, dest_file, e
        ),
    }

    let mut log_config_file = PathBuf::new();
    log_config_file.push(&cargo_manifest_dir);
    log_config_file.push("config");
    log_config_file.push("log4rs.yml");

    let log_dest_file = Path::new(exe_dir).join("log4rs.yml");

    println!(
        "BUILD: Attempting to move {:?} to {:?}",
        log_config_file, log_dest_file
    );

    match fs::copy(&log_config_file, &log_dest_file) {
        Ok(_) => println!(
            "BUILD: Copied log config file successfully from {:?} to {:?}",
            log_config_file, log_dest_file
        ),
        Err(e) => println!(
            "BUILD: Failed to copy log config file from {:?} to {:?}: {:?}",
            log_config_file, log_dest_file, e
        ),
    }
}
