use std::fs;
use std::process;
use toml::Value;

fn main() {
    println!("Checking that sway-lib-std version matches Cargo.toml");

    let workspace_root = std::env::current_dir().expect("Failed to get current directory");
    let cargo_content = fs::read_to_string(workspace_root.join("Cargo.toml"))
        .expect("Failed to read Cargo.toml");
    let forc_content = fs::read_to_string(workspace_root.join("sway-lib-std/Forc.toml"))
        .expect("Failed to read sway-lib-std/Forc.toml");

    let cargo_toml: Value = cargo_content.parse()
        .expect("Failed to parse Cargo.toml");
    let forc_toml: Value = forc_content.parse()
        .expect("Failed to parse Forc.toml");

    let cargo_version = cargo_toml["workspace"]["package"]["version"]
        .as_str()
        .expect("Could not find version in Cargo.toml");

    let forc_version = forc_toml["project"]["version"]
        .as_str()
        .expect("Could not find version in sway-lib-std/Forc.toml");

    if cargo_version != forc_version {
        eprintln!("Version mismatch!");
        eprintln!("Cargo.toml: {}", cargo_version);
        eprintln!("sway-lib-std/Forc.toml: {}", forc_version);
        process::exit(1);
    }

    println!("Versions match: {}", cargo_version);
    process::exit(0);
}
