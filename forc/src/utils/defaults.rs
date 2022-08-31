/// We intentionally don't construct this using [serde]'s default deserialization so we get
/// the chance to insert some helpful comments and nicer formatting.
pub(crate) fn default_manifest(project_name: &str, entry_type: &str) -> String {
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

/// Creates a default Cargo manifest for the Rust-based tests.
/// It includes necessary packages to make the Rust-based
/// tests work.
pub(crate) fn default_tests_manifest(project_name: &str) -> String {
    let author = get_author();

    format!(
        r#"[project]
name = "{project_name}"
version = "0.1.0"
authors = ["{author}"]
edition = "2021"
license = "Apache-2.0"

[dependencies]
fuels = {{ version = "0.21", features = ["fuel-core-lib"] }}
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
        "{}{}{}{}{}{}{}",
        r#"use fuels::{prelude::*, tx::ContractId};

// Load abi from json
abigen!(MyContract, "out/debug/"#,
        project_name,
        r#"-abi.json");

async fn get_contract_instance() -> (MyContract, ContractId) {
    // Launch a local network and deploy the contract
    let mut wallets = launch_custom_provider_and_get_wallets(
        WalletsConfig::new(
            Some(1),             /* Single wallet */
            Some(1),             /* Single coin (UTXO) */
            Some(1_000_000_000), /* Amount per coin */
        ),
        None,
    )
    .await;
    let wallet = wallets.pop().unwrap();

    let id = Contract::deploy(
        "./out/debug/"#,
        project_name,
        r#".bin",
        &wallet,
        TxParameters::default(),
        StorageConfiguration::with_storage_path(Some(
            "./out/debug/"#,
        project_name,
        r#"-storage_slots.json".to_string(),
        )),
    )
    .await
    .unwrap();

    let instance = MyContractBuilder::new(id.to_string(), wallet).build();

    (instance, id.into())
}

#[tokio::test]
async fn can_get_contract_id() {
    let (_instance, _id) = get_contract_instance().await;

    // Now you have an instance of your contract you can use to test each function
}
"#
    )
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
fn parse_default_manifest() {
    use sway_utils::constants::MAIN_ENTRY;
    tracing::info!(
        "{:#?}",
        toml::from_str::<forc_pkg::Manifest>(&default_manifest("test_proj", MAIN_ENTRY)).unwrap()
    )
}

#[test]
fn parse_default_tests_manifest() {
    tracing::info!(
        "{:#?}",
        toml::from_str::<forc_pkg::Manifest>(&default_tests_manifest("test_proj")).unwrap()
    )
}
