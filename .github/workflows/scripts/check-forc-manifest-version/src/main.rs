use std::fs;
use std::process::exit;
use toml::Value;

fn main() {
    println!("Checking that sway-lib-std version matches Cargo.toml");

    let cargo_content = fs::read_to_string("Cargo.toml")
        .expect("Failed to read Cargo.toml");
    let forc_content = fs::read_to_string("sway-lib-std/Forc.toml")
        .expect("Failed to read sway-lib-std/Forc.toml");

    let cargo_toml: Value = cargo_content.parse()
        .expect("Failed to parse Cargo.toml");
    let forc_toml: Value = forc_content.parse()
        .expect("Failed to parse Forc.toml");

    let cargo_version = cargo_toml["package"]["version"]
        .as_str()
        .or_else(|| cargo_toml["workspace"]["package"]["version"].as_str())
        .expect("Could not find version in Cargo.toml");

    let forc_version = forc_toml["project"]["version"]
        .as_str()
        .expect("Could not find version in Forc.toml");

    if cargo_version != forc_version {
        eprintln!("Version mismatch!");
        eprintln!("Cargo.toml: {}", cargo_version);
        eprintln!("sway-lib-std/Forc.toml: {}", forc_version);
        process::exit(1);
    }

    println!("Versions match: {}", cargo_version);
    process::exit(0);
}
