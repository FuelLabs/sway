//! This module handles everything to do with index files.
//!
//! Index files are for creating set of information for identifying a published
//! package. They are used by forc while fetching to actually convert a registry
//! index into a IPFS CID. We also add some metadata to this index files to
//! enable forc to do "more clever" fetching during build process. By moving
//! dependency resolution from the time a package is fetched to the point we
//! start fetching we are actively enabling forc to fetch packages and their
//! dependencies in parallel.
//!
//! There are two main things forc needs to be able to do for index files:
//!   1: Creation of index files from published packages
//!   2: Calculating correct path for given package index.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize)]
pub struct IndexFile {
    /// Each published instance for this specific package, keyed by their
    /// versions. The reason we are doing this type of mapping is for use of
    /// ease and deterministic ordering, we are effectively duplicating version
    /// of package but keeping `PackageEntry` self contained.
    #[serde(flatten)]
    versions: BTreeMap<semver::Version, PackageEntry>,
}

/// A unique representation of each published package to `forc.pub`. Contains:
///
/// 1. The name of the package.
/// 2. The version of the package.
/// 3. CID of the package's source code. This is how forc actually resolves a
///    package name, version information into actual information on how to get
///    the package.
/// 4. CID of the package's abi if the package is a contract.
/// 5. Dependencies of this package. If there are other packages this package
///    depends on, some information can be directly found in the root package
///    to enable parallel fetching.
#[derive(Serialize, Deserialize, Clone)]
pub struct PackageEntry {
    /// Name of the package.
    /// This is the actual package name needed in forc.toml file to fetch this
    /// package.
    #[serde(alias = "package_name")]
    name: String,
    /// Version of the package.
    /// This is the actual package version needed in forc.toml file to fetch
    /// this package.
    version: semver::Version,
    /// IPFS CID of this specific package's source code. This is pinned by
    /// forc.pub at the time of package publishing and thus will be
    /// available all the time.
    source_cid: String,
    /// IPFS CID of this specific package's abi. This is pinned by
    /// forc.pub at the time of package publishing and thus will be
    /// available all the time if this exists in the first place, i.e the
    /// package is a contract.
    abi_cid: Option<String>,
    /// Dependencies of the current package entry. Can be consumed to enable
    /// parallel fetching by the consumers of this index, mainly forc.
    dependencies: Vec<PackageDependencyIdentifier>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PackageDependencyIdentifier {
    /// Name of the dependency.
    /// Name and version information can be used by consumer of this index
    /// to resolve dependencies.
    package_name: String,
    /// Version of the dependency.
    /// Name and version information can be used by consumer of this index
    /// to resolve dependencies.
    version: String,
}

impl PackageEntry {
    pub fn new(
        name: String,
        version: semver::Version,
        source_cid: String,
        abi_cid: Option<String>,
        dependencies: Vec<PackageDependencyIdentifier>,
    ) -> Self {
        Self {
            name,
            version,
            source_cid,
            abi_cid,
            dependencies,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> &semver::Version {
        &self.version
    }

    pub fn source_cid(&self) -> &str {
        &self.source_cid
    }

    pub fn abi_cid(&self) -> Option<&str> {
        self.abi_cid.as_deref()
    }

    pub fn dependencies(&self) -> impl Iterator<Item = &PackageDependencyIdentifier> {
        self.dependencies.iter()
    }
}

impl IndexFile {
    pub fn get(&self, version: &semver::Version) -> Option<&PackageEntry> {
        self.versions.get(version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_deserialize_empty_index() {
        let index = IndexFile {
            versions: BTreeMap::new(),
        };

        let serialized = serde_json::to_string(&index).unwrap();
        assert_eq!(serialized, "{}");
        let deserialized: IndexFile = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.versions.len(), 0);
    }

    #[test]
    fn test_json_format() {
        // Test parsing from a JSON
        let json = r#"{
        "0.0.1":{
            "package_name":"tester",
            "version":"0.0.1",
            "source_cid":"QmOlderHash",
            "abi_cid":"QmOlderAbiHash",
            "dependencies":[]
        },
        "0.0.2":{
            "package_name":"tester",
            "version":"0.0.2",
            "source_cid":"QmExampleHash",
            "abi_cid":"QmExampleAbiHash",
            "dependencies":[]
        }
    }"#;

        let deserialized: IndexFile = serde_json::from_str(json).unwrap();

        assert_eq!(deserialized.versions.len(), 2);
        assert!(deserialized
            .versions
            .contains_key(&semver::Version::new(0, 0, 1)));
        assert!(deserialized
            .versions
            .contains_key(&semver::Version::new(0, 0, 2)));

        let v011 = &deserialized.versions[&semver::Version::new(0, 0, 1)];
        assert_eq!(v011.source_cid, "QmOlderHash");
        assert_eq!(v011.abi_cid, Some("QmOlderAbiHash".to_string()));
        assert_eq!(v011.dependencies.len(), 0);

        let v012 = &deserialized.versions[&semver::Version::new(0, 0, 2)];
        assert_eq!(v012.source_cid, "QmExampleHash");
        assert_eq!(v012.abi_cid, Some("QmExampleAbiHash".to_string()));
        assert_eq!(v012.dependencies.len(), 0);
    }

    #[test]
    fn test_json_with_dependencies() {
        // Test parsing a JSON with dependencies
        let json = r#"{
            "1.0.0": {
                "package_name": "main-package",
                "version": "1.0.0",
                "source_cid": "QmMainHash",
                "abi_cid": null,
                "dependencies": [
                    {
                        "package_name": "dep-package",
                        "version": "^0.5.0"
                    },
                    {
                        "package_name": "another-dep",
                        "version": "=0.9.1"
                    },
                    {
                        "package_name": "third-dep",
                        "version": "0.2.0"
                    }
                ]
            }
        }"#;

        let deserialized: IndexFile = serde_json::from_str(json).unwrap();

        // Verify main package
        assert_eq!(deserialized.versions.len(), 1);
        assert!(deserialized
            .versions
            .contains_key(&semver::Version::new(1, 0, 0)));

        let main_pkg = &deserialized.versions[&semver::Version::new(1, 0, 0)];
        assert_eq!(main_pkg.name, "main-package");
        assert_eq!(main_pkg.source_cid, "QmMainHash");
        assert_eq!(main_pkg.abi_cid, None);

        // Verify dependencies
        assert_eq!(main_pkg.dependencies.len(), 3);

        // Check first dependency
        let dep1 = &main_pkg.dependencies[0];
        assert_eq!(dep1.package_name, "dep-package");
        assert_eq!(dep1.version, "^0.5.0");

        // Check second dependency
        let dep2 = &main_pkg.dependencies[1];
        assert_eq!(dep2.package_name, "another-dep");
        assert_eq!(dep2.version, "=0.9.1");

        // Check third dependency
        let dep3 = &main_pkg.dependencies[2];
        assert_eq!(dep3.package_name, "third-dep");
        assert_eq!(dep3.version, "0.2.0");

        // Test round-trip serialization
        let serialized = serde_json::to_string_pretty(&deserialized).unwrap();
        println!("Re-serialized JSON: {}", serialized);

        // Deserialize again to ensure it's valid
        let re_deserialized: IndexFile = serde_json::from_str(&serialized).unwrap();
        assert_eq!(re_deserialized.versions.len(), 1);

        // Verify the structure is preserved
        let main_pkg2 = &re_deserialized.versions[&semver::Version::new(1, 0, 0)];
        assert_eq!(main_pkg2.dependencies.len(), 3);
    }

    #[test]
    fn test_json_with_missing_optional_fields() {
        // Test parsing a JSON where some optional fields are missing
        let json = r#"{
            "0.5.0": {
                "package_name": "minimal-package",
                "version": "0.5.0",
                "source_cid": "QmMinimalHash",
                "dependencies": []
            }
        }"#;

        let deserialized: IndexFile = serde_json::from_str(json).unwrap();

        assert_eq!(deserialized.versions.len(), 1);
        let pkg = &deserialized.versions[&semver::Version::new(0, 5, 0)];
        assert_eq!(pkg.name, "minimal-package");
        assert_eq!(pkg.source_cid, "QmMinimalHash");
        assert_eq!(pkg.abi_cid, None);
        assert_eq!(pkg.dependencies.len(), 0);
    }

    #[test]
    fn test_package_entry_new() {
        // Test the new() constructor method
        let dependencies = vec![PackageDependencyIdentifier {
            package_name: "dep1".to_string(),
            version: "^1.0".to_string(),
        }];

        let entry = PackageEntry::new(
            "test-package".to_string(),
            semver::Version::new(2, 0, 0),
            "QmTestHash".to_string(),
            Some("QmAbiHash".to_string()),
            dependencies.clone(),
        );

        assert_eq!(entry.name, "test-package");
        assert_eq!(entry.version, semver::Version::new(2, 0, 0));
        assert_eq!(entry.source_cid, "QmTestHash");
        assert_eq!(entry.abi_cid, Some("QmAbiHash".to_string()));
        assert_eq!(entry.dependencies.len(), 1);
        assert_eq!(entry.dependencies[0].package_name, "dep1");
        assert_eq!(entry.dependencies[0].version, "^1.0");
    }
}
