use regex::Regex;
use std::collections::HashSet; // Import HashSet
use std::fs;
use std::process::exit;
use toml::Value;

// Dependency name required to use x.y.z format
const REQUIRED_XYZ_DEP: &str = "fuel-core-client";

// Dependency names allowed (but not required) to use x.y.z format
// Add names of common dev-dependencies here if you want to allow x.y.z for them
const ALLOWED_XYZ_DEPS: &[&str] = &["etk-asm", "etk-ops", "dap", "fuel-abi-types"];

// Regex to strictly match semantic version x.y.z (no prefixes like ^, ~)
const XYZ_REGEX_STR: &str = r"^\d+\.\d+\.\d+([\w.-]*)$"; // Allow suffixes like -alpha, .1
                                                         // Regex to strictly match semantic version x.y (no prefixes)
const STRICT_XYZ_REQ_REGEX_STR: &str = r"^=\s*\d+\.\d+\.\d+([\w.-]*)$";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cargo_toml_path = args.get(1).unwrap_or_else(|| {
        eprintln!("Usage: check-dep-versions <path/to/root/Cargo.toml>");
        exit(1);
    });

    println!("Checking dependency versions in: {}", cargo_toml_path);

    let content = fs::read_to_string(cargo_toml_path).expect("Failed to read root Cargo.toml");
    let doc: Value = content
        .parse::<Value>()
        .expect("Failed to parse root Cargo.toml");

    let xyz_regex = Regex::new(XYZ_REGEX_STR).unwrap();
    let strict_xyz_req_regex = Regex::new(STRICT_XYZ_REQ_REGEX_STR).unwrap();

    // Convert the allowlist slice to a HashSet for efficient lookups
    let allowed_xyz_set: HashSet<String> = ALLOWED_XYZ_DEPS.iter().map(|s| s.to_string()).collect();

    let mut errors_found = false;

    if let Some(workspace) = doc.get("workspace") {
        if let Some(dependencies) = workspace.get("dependencies") {
            if let Some(deps_table) = dependencies.as_table() {
                for (name, value) in deps_table {
                    if let Some(table) = value.as_table() {
                        if table.contains_key("path") {
                            continue;
                        }
                    }

                    let version_str = match value {
                        Value::String(s) => Some(s.as_str()),
                        Value::Table(t) => t.get("version").and_then(|v| v.as_str()),
                        _ => None,
                    };

                    if let Some(version) = version_str {
                        let version_trimmed = version.trim(); // Trim whitespace

                        // Check if the version string matches x.y.z patterns
                        let is_specific_xyz = xyz_regex.is_match(version_trimmed)
                            || strict_xyz_req_regex.is_match(version_trimmed);

                        // Check required dependency
                        if name == REQUIRED_XYZ_DEP {
                            if !is_specific_xyz {
                                eprintln!(
                                    "Error: Dependency '{}' MUST use specific 'x.y.z' format (e.g., \"0.41.9\" or \"= 0.41.9\"), but found '{}'",
                                    name, version
                                );
                                errors_found = true;
                            }
                        } else if allowed_xyz_set.contains(name) {
                            // It's on the allowlist, so x.y.z OR x.y is fine. No check needed here for format.
                            continue;
                        } else {
                            // Check all other dependencies (not required and not allowed x.y.z explicitly)
                            if is_specific_xyz {
                                eprintln!(
                                    "Error: Dependency '{}' uses specific 'x.y.z' format ('{}'). Use 'x.y' format (e.g., \"0.59\") or add to allowlist if it's a dev-dependency.",
                                    name, version
                                );
                                errors_found = true;
                            }
                        }
                    }
                }
            } else {
                eprintln!("Warning: [workspace.dependencies] is not a table.");
            }
        } else {
            println!("No [workspace.dependencies] section found.");
        }
    } else {
        eprintln!("Warning: No [workspace] section found in Cargo.toml.");
    }

    if errors_found {
        eprintln!("\nDependency version format check failed.");
        exit(1);
    } else {
        println!("\nDependency version format check passed.");
    }
}
