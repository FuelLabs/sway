use std::fs;

fn main() {
    let cargo_toml_value = fs::read_to_string("Cargo.toml")
        .ok()
        .and_then(|cargo_toml_str| cargo_toml_str.parse::<toml::Value>().ok());
    if let Some(cargo_toml) = cargo_toml_value {
        let package = cargo_toml.get("package");
        let version = package
            .and_then(|package| package.get("version"))
            .map(|version| version.to_string());

        if let Some(version) = version {
            // drop `"` around version
            let mut version = version.chars();
            version.next();
            version.next_back();
            // We found the version write it to a file
            fs::write(".version", version.as_str())
                .unwrap_or_else(|_| panic!("Couldn't write version to .version"));
        }
    }
}
