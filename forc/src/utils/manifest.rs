use crate::utils::dependency::Dependency;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use sway_utils::constants::DEFAULT_NODE_URL;

// using https://github.com/rust-lang/cargo/blob/master/src/cargo/util/toml/mod.rs as the source of
// implementation strategy

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Manifest {
    pub project: Project,
    pub network: Option<Network>,
    pub dependencies: Option<BTreeMap<String, Dependency>>,
}

impl Manifest {}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Project {
    #[deprecated = "use the authors field instead, the author field will be removed soon."]
    pub author: Option<String>,
    pub authors: Option<Vec<String>>,
    pub name: String,
    pub organization: Option<String>,
    pub license: String,
    #[serde(default = "default_entry")]
    pub entry: String,
}

fn default_entry() -> String {
    "main.sw".into()
}

#[derive(Serialize, Deserialize, Debug)]
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
        toml::from_str::<Manifest>(&super::defaults::default_manifest("test_proj")).unwrap()
    )
}
