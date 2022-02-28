/// We intentionally don't construct this using [serde]'s default deserialization so we get
/// the chance to insert some helpful comments and nicer formatting.
pub(crate) fn default_manifest(project_name: &str) -> String {
    let real_name = whoami::realname();

    format!(
        r#"[project]
authors = ["{real_name}"]
entry = "main.sw"
license = "Apache-2.0"
name = "{project_name}"

[dependencies]
core = {{ git = "http://github.com/FuelLabs/sway-lib-core" }}
std = {{ git = "http://github.com/FuelLabs/sway-lib-std" }}
"#
    )
}

/// Creates a default Cargo manifest for the Rust-based tests.
/// It includes necessary packages to make the Rust-based
/// tests work, such as the abigen macro, fuels-rs, and
/// the fuel client.
pub(crate) fn default_tests_manifest(project_name: &str) -> String {
    let real_name = whoami::realname();

    format!(
        r#"[package]
authors = ["{real_name}"]
edition = "2021"
license = "Apache-2.0"
name = "{project_name}"
version = "0.1.0"

[dependencies]
fuel-gql-client = {{ version = "0.2", default-features = false }}
fuel-tx = "0.3"
fuels-abigen-macro = "0.3"
fuels-contract = "0.3"
fuels-core = "0.3"
rand = "0.8"
tokio = {{ version = "1.12", features = ["rt", "macros"] }}

[[test]]
harness = true
name = "integration_tests"
path = "tests/harness.rs"
"#
    )
}

pub(crate) fn default_program() -> String {
    r#"script;

fn main() {

}
"#
    .into()
}

pub(crate) fn default_test_program() -> String {
    r#"

#[tokio::test]
async fn harness() {
    assert_eq!(true, true);
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
