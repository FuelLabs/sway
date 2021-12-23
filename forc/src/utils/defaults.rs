/// We intentionally don't construct this using [serde]'s default deserialization so we get
/// the chance to insert some helpful comments and nicer formatting.
pub(crate) fn default_manifest(project_name: &str) -> String {
    let real_name = whoami::realname();

    format!(
        r#"[project]
name = "{}"
author  = "{}"
entry = "main.sw"
license = "Apache-2.0"
"#,
        real_name, project_name,
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
name = "{}"
version = "0.1.0"
author  = "{}"
edition = "2021"
license = "Apache-2.0"

[dependencies]
tokio = {{ version = "1.12", features = ["rt", "macros"] }}
fuels-abigen-macro = {{ git = "ssh://git@github.com/FuelLabs/fuels-rs.git" }}
fuels-rs = {{ git = "ssh://git@github.com/FuelLabs/fuels-rs.git" }}
fuel-client = {{ git = "ssh://git@github.com/FuelLabs/fuel-core", default-features = false }}
fuel-tx = {{ git = "ssh://git@github.com/FuelLabs/fuel-tx.git" }}
rand = "0.8"

[[test]]
name = "integration_tests"
path = "tests/harness.rs"
harness = true
"#,
        real_name, project_name,
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
