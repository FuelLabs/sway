/// We intentionally don't construct this using [serde]'s default deserialization so we get
/// the chance to insert some helpful comments and nicer formatting.
pub(crate) fn default_manifest(project_name: &str, entry_type: &str) -> String {
    let real_name = whoami::realname();

    format!(
        r#"[project]
authors = ["{real_name}"]
entry = "{entry_type}"
license = "Apache-2.0"
name = "{project_name}"

[dependencies]
"#
    )
}

/// Creates a default Cargo manifest for the Rust-based tests.
/// It includes necessary packages to make the Rust-based
/// tests work.
pub(crate) fn default_tests_manifest(project_name: &str) -> String {
    let real_name = whoami::realname();

    format!(
        r#"[project]
authors = ["{real_name}"]
edition = "2021"
license = "Apache-2.0"
name = "{project_name}"
version = "0.1.0"

[dependencies]
fuel-gql-client = {{ version = "0.6", default-features = false }}
fuel-tx = "0.9"
fuels-abigen-macro = "0.10"
fuels = "0.10"
rand = "0.8"
tokio = {{ version = "1.12", features = ["rt", "macros"] }}

[[test]]
harness = true
name = "integration_tests"
path = "tests/harness.rs"
"#
    )
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

fn main() {

}
"#
    .into()
}

pub(crate) fn default_library(project_name: &str) -> String {
    format!(
        "library {project_name};

// anything `pub` here will be exported as a part of this library's API
"
    )
}

pub(crate) fn default_predicate() -> String {
    r#"predicate;

fn main() -> bool {
    false
}
"#
    .into()
}

// TODO Ideally after (instance, id) it should link to the The Fuels-rs Book
// to provide further information for writing tests/working with sway
pub(crate) fn default_test_program(project_name: &str) -> String {
    format!(
        "{}{}{}{}{}",
        r#"use fuel_tx::{ContractId, Salt};
use fuels_abigen_macro::abigen;
use fuels::prelude::*;
use fuels::test_helpers;

// Load abi from json
abigen!(MyContract, "out/debug/"#,
        project_name,
        r#"-abi.json");

async fn get_contract_instance() -> (MyContract, ContractId) {
    // Deploy the compiled contract
    let salt = Salt::from([0u8; 32]);
    let compiled = Contract::load_sway_contract("./out/debug/"#,
        project_name,
        r#".bin", salt).unwrap();

    // Launch a local network and deploy the contract
    let (provider, wallet) = test_helpers::setup_test_provider_and_wallet().await;

    let id = Contract::deploy(&compiled, &provider, &wallet, TxParameters::default())
        .await
        .unwrap();

    let instance = MyContract::new(id.to_string(), provider, wallet);

    (instance, id)
}

#[tokio::test]
async fn can_get_contract_id() {
    let (_instance, _id) = get_contract_instance().await;

    // Now you have an instance of your contract you can use to test each function
}"#
    )
}

pub(crate) fn default_gitignore() -> String {
    r#"out
target
"#
    .into()
}

#[test]
fn parse_default_manifest() {
    use sway_utils::constants::MAIN_ENTRY;
    println!(
        "{:#?}",
        toml::from_str::<forc_pkg::Manifest>(&default_manifest("test_proj", MAIN_ENTRY)).unwrap()
    )
}

#[test]
fn parse_default_tests_manifest() {
    println!(
        "{:#?}",
        toml::from_str::<forc_pkg::Manifest>(&default_tests_manifest("test_proj")).unwrap()
    )
}
