use std::fs;
use std::path::{Path, PathBuf};

fn minify_json(json: &str) -> String {
    let mut result = String::with_capacity(json.len());
    let mut in_string = false;
    let mut previous_char: Option<char> = None;

    for c in json.chars() {
        if in_string {
            result.push(c);
            if c == '"' && previous_char != Some('\\') {
                in_string = false;
            }
        } else {
            match c {
                '"' => {
                    result.push(c);
                    in_string = true;
                }
                ' ' | '\n' | '\r' | '\t' => continue, // Skip whitespace
                _ => result.push(c),
            }
        }
        previous_char = Some(c);
    }
    result
}

fn update_proxy_abi_decl_with_file(source_file_path: &Path, minified_json: &str) {
    // Read the contents of the source file
    let mut source_code = fs::read_to_string(source_file_path).expect("Unable to read source file");

    // Prepare the replacement string for the `abigen!` macro
    let escaped_json = minified_json.replace('\\', "\\\\").replace('"', "\\\"");
    let new_abigen =
        format!("abigen!(Contract(name = \"ProxyContract\", abi = \"{escaped_json}\",));");

    // Use a regular expression to find and replace the `abigen!` macro
    let re = regex::Regex::new(r#"abigen!\(Contract\(name = "ProxyContract", abi = ".*?",\)\);"#)
        .expect("Invalid regex pattern");

    // Replace the existing `abigen!` macro with the new one containing the updated ABI
    if re.is_match(&source_code) {
        source_code = re.replace(&source_code, new_abigen.as_str()).to_string();
    } else {
        panic!("abigen! macro not found in the source file");
    }

    // Write the modified source code back to the source file
    fs::write(source_file_path, source_code).expect("Unable to write back to the source file");
}

fn main() {
    // Path to the JSON file in the root directory next to the `src` folder
    let json_path = PathBuf::from("proxy_abi/proxy_contract-abi.json");
    // Read the contents of the JSON file
    let json_content =
        fs::read_to_string(json_path).expect("Unable to read proxy_contract-abi.json");

    // Minify the JSON content
    let minified_json = minify_json(&json_content);

    // If proxy_contract-abi.json is changed, re-run this script
    println!("cargo:rerun-if-changed=proxy_abi/proxy_contract-abi.json");

    // Path to the Rust source file that contains the `abigen!` macro that
    // creates a `ProxyContract`.
    let util_tx_path = PathBuf::from("src/util/tx.rs");
    update_proxy_abi_decl_with_file(&util_tx_path, &minified_json);

    let test_path = PathBuf::from("tests/deploy.rs");
    update_proxy_abi_decl_with_file(&test_path, &minified_json);

    let deploy_path = PathBuf::from("src/op/deploy.rs");
    update_proxy_abi_decl_with_file(&deploy_path, &minified_json);
}
