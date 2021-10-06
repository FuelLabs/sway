use crate::utils::dependency::Dependency;
use serde::Deserialize;
use std::collections::BTreeMap;

use super::constants::DEFAULT_NODE_URL;

// using https://github.com/rust-lang/cargo/blob/master/src/cargo/util/toml/mod.rs as the source of
// implementation strategy

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Manifest {
    pub project: Project,
    pub network: Option<Network>,
    pub tx_inputs: Vec<fuel_tx::Input>,
    pub dependencies: Option<BTreeMap<String, Dependency>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Project {
    pub author: String,
    pub name: String,
    pub license: String,
    #[serde(default = "default_entry")]
    pub entry: String,
}

fn default_entry() -> String {
    "main.sw".into()
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Network {
    #[serde(default = "default_url")]
    pub url: String,
}

fn default_url() -> String {
    DEFAULT_NODE_URL.into()
}

#[test]
fn try_parse() {
    println!(
        "{:#?}",
        toml::from_str::<Manifest>(&super::defaults::default_manifest("test_proj".into())).unwrap()
    )
}
