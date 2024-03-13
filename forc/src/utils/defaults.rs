/// We intentionally don't construct this using [serde]'s default deserialization so we get
/// the chance to insert some helpful comments and nicer formatting.
pub(crate) fn default_pkg_manifest(project_name: &str, entry_type: &str) -> String {
    let author = get_author();

    format!(
        r#"[project]
authors = ["{author}"]
entry = "{entry_type}"
license = "Apache-2.0"
name = "{project_name}"

[dependencies]
"#
    )
}

pub(crate) fn default_workspace_manifest() -> String {
    r#"[workspace]
members = []"#
        .to_string()
}

pub(crate) fn default_contract() -> String {
    r#"contract;

abi MyContract {
    fn test_function() -> bool;
}

impl MyContract for Contract {
    fn test_function() -> bool {
        true
    }
}
"#
    .into()
}

pub(crate) fn default_script() -> String {
    r#"script;

use std::logging::log;

configurable {
    SECRET_NUMBER: u64 = 0
}

fn main() -> u64 {
    log(SECRET_NUMBER);
    return SECRET_NUMBER;
}
"#
    .into()
}

pub(crate) fn default_library() -> String {
    "library;

// anything `pub` here will be exported as a part of this library's API
"
    .into()
}

pub(crate) fn default_predicate() -> String {
    r#"predicate;

fn main() -> bool {
    true
}
"#
    .into()
}

pub(crate) fn default_gitignore() -> String {
    r#"out
target
"#
    .into()
}

fn get_author() -> String {
    std::env::var(sway_utils::FORC_INIT_MANIFEST_AUTHOR).unwrap_or_else(|_| whoami::realname())
}

#[test]
fn parse_default_pkg_manifest() {
    use sway_utils::constants::MAIN_ENTRY;
    tracing::info!(
        "{:#?}",
        toml::from_str::<forc_pkg::PackageManifest>(&default_pkg_manifest("test_proj", MAIN_ENTRY))
            .unwrap()
    )
}
#[test]
fn parse_default_workspace_manifest() {
    tracing::info!(
        "{:#?}",
        toml::from_str::<forc_pkg::PackageManifest>(&default_workspace_manifest()).unwrap()
    )
}
