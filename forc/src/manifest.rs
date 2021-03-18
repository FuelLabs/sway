use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// using https://github.com/rust-lang/cargo/blob/master/src/cargo/util/toml/mod.rs as the source of
// implementation strateby

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Manifest {
    pub project: Project,
    pub dependencies: Option<BTreeMap<String, Dependency>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Project {
    pub author: String,
    pub name: String,
    pub license: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Dependency {
    /// In the simple format, only a version is specified, eg.
    /// `package = "<version>"`
    Simple(String),
    /// The simple format is equivalent to a detailed dependency
    /// specifying only a version, eg.
    /// `package = { version = "<version>" }`
    Detailed(DependencyDetails),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct DependencyDetails {
    pub(crate) version: Option<String>,
    pub(crate) path: Option<String>,
    pub(crate) git: Option<String>,
}

#[test]
fn try_parse() {
    println!(
        "{:#?}",
        toml::from_str::<Manifest>(&crate::defaults::default_manifest()).unwrap()
    )
}
