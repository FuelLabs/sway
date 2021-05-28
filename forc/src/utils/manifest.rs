use serde::Deserialize;
use std::collections::BTreeMap;

// using https://github.com/rust-lang/cargo/blob/master/src/cargo/util/toml/mod.rs as the source of
// implementation strategy

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Manifest {
    pub project: Project,
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

#[derive(Deserialize, Debug)]
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
        toml::from_str::<Manifest>(&super::defaults::default_manifest("test_proj".into())).unwrap()
    )
}
