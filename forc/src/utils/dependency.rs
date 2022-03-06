use crate::utils::manifest::Manifest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// A collection of remote dependency related functions

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
    pub(crate) branch: Option<String>,
    pub(crate) tag: Option<String>,
}
pub enum OfflineMode {
    Yes,
    No,
}

impl From<bool> for OfflineMode {
    fn from(v: bool) -> OfflineMode {
        match v {
            true => OfflineMode::Yes,
            false => OfflineMode::No,
        }
    }
}

// Helper to get only detailed dependencies (`Dependency::Detailed`).
pub fn get_detailed_dependencies(manifest: &mut Manifest) -> HashMap<String, &DependencyDetails> {
    let mut dependencies: HashMap<String, &DependencyDetails> = HashMap::new();

    if let Some(ref mut deps) = manifest.dependencies {
        for (dep_name, dependency_details) in deps.iter_mut() {
            match dependency_details {
                Dependency::Simple(..) => continue,
                Dependency::Detailed(dep_details) => {
                    dependencies.insert(dep_name.to_owned(), dep_details)
                }
            };
        }
    }

    dependencies
}
