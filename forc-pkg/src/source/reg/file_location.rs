use serde::{Deserialize, Serialize};
use std::{fmt::Display, path::PathBuf};

/// Number of levels of nesting to use for file locations.
const NESTING_LEVELS: usize = 2;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum Namespace {
    /// Flat namespace means no sub-namespace with different domains.
    /// Location calculator won't be adding anything specific for this to the
    /// file location.
    Flat,
    /// Domain namespace means we have custom namespaces and first component of
    /// the file location of the index file will be the domain of the namespace.
    /// Which means in the index repository all namespaced packages will first
    /// have the namespace in their paths.
    Domain(String),
}

impl Display for Namespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Namespace::Flat => write!(f, ""),
            Namespace::Domain(s) => write!(f, "{s}"),
        }
    }
}

/// Calculates the exact file location from the root of the namespace repo.
/// If the configuration includes a namespace, it will be the first part of
/// the path followed by chunks.
pub fn location_from_root(chunk_size: usize, namespace: &Namespace, package_name: &str) -> PathBuf {
    let mut path = PathBuf::new();

    // Add domain to path if namespace is 'Domain' and it is not empty
    // otherwise skip.
    match namespace {
        Namespace::Domain(domain) if !domain.is_empty() => {
            path.push(domain);
        }
        _ => {}
    }

    // If chunking is disabled we do not have any folder in the index.
    if chunk_size == 0 {
        path.push(package_name);
        return path;
    }

    let char_count = chunk_size * NESTING_LEVELS;
    let to_be_chunked_section = package_name
        .chars()
        .enumerate()
        .take_while(|(index, _)| *index < char_count)
        .map(|(_, ch)| ch);

    let chars: Vec<char> = to_be_chunked_section.collect();
    for chunk in chars.chunks(chunk_size) {
        let chunk_str: String = chunk.iter().collect();
        path.push(chunk_str);
    }

    path.push(package_name);
    path
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::reg::index_file::PackageEntry;
    use semver::Version;
    use std::path::Path;

    fn create_package_entry(name: &str) -> PackageEntry {
        let name = name.to_string();
        let version = Version::new(1, 0, 0);
        let source_cid = "QmHash".to_string();
        let abi_cid = None;
        let dependencies = vec![];
        let yanked = false;
        PackageEntry::new(name, version, source_cid, abi_cid, dependencies, yanked)
    }

    #[test]
    fn test_flat_namespace_with_small_package() {
        let chunk_size = 2;
        let namespace = Namespace::Flat;
        let entry = create_package_entry("ab");

        let path = location_from_root(chunk_size, &namespace, entry.name());

        assert_eq!(path, Path::new("ab").join("ab"));
    }

    #[test]
    fn test_flat_namespace_with_regular_package() {
        let chunk_size = 2;
        let namespace = Namespace::Flat;
        let entry = create_package_entry("foobar");

        let path = location_from_root(chunk_size, &namespace, entry.name());

        // Should produce: fo/ob/foobar
        assert_eq!(path, Path::new("fo").join("ob").join("foobar"));
    }

    #[test]
    fn test_domain_namespace() {
        let chunk_size = 2;
        let namespace = Namespace::Domain("example".to_string());
        let entry = create_package_entry("foobar");

        let path = location_from_root(chunk_size, &namespace, entry.name());

        // Should produce: example/fo/ob/foobar
        assert_eq!(
            path,
            Path::new("example").join("fo").join("ob").join("foobar")
        );
    }

    #[test]
    fn test_odd_length_package_name() {
        let chunk_size = 2;
        let namespace = Namespace::Flat;
        let entry = create_package_entry("hello");

        let path = location_from_root(chunk_size, &namespace, entry.name());

        // Should produce: he/ll/hello
        assert_eq!(path, Path::new("he").join("ll").join("hello"));
    }

    #[test]
    fn test_larger_chunking_size() {
        let chunk_size = 3;
        let namespace = Namespace::Flat;
        let entry = create_package_entry("fibonacci");

        let path = location_from_root(chunk_size, &namespace, entry.name());

        // Should produce: fib/ona/fibonacci
        assert_eq!(path, Path::new("fib").join("ona").join("fibonacci"));
    }

    #[test]
    fn test_chunking_size_larger_than_name() {
        let chunk_size = 10;
        let namespace = Namespace::Flat;
        let entry = create_package_entry("small");

        let path = location_from_root(chunk_size, &namespace, entry.name());

        // Should produce: small/small
        assert_eq!(path, Path::new("small").join("small"));
    }

    #[test]
    fn test_unicode_package_name() {
        let chunk_size = 2;
        let namespace = Namespace::Flat;
        let entry = create_package_entry("héllo");

        let path = location_from_root(chunk_size, &namespace, entry.name());

        // Should produce: hé/ll/héllo
        assert_eq!(path, Path::new("hé").join("ll").join("héllo"));
    }

    #[test]
    fn test_empty_package_name() {
        let chunk_size = 0;
        let namespace = Namespace::Flat;
        let entry = create_package_entry("");

        let path = location_from_root(chunk_size, &namespace, entry.name());

        // Should just produce: ""
        assert_eq!(path, Path::new(""));
    }

    #[test]
    fn test_chunking_size_zero() {
        let chunk_size = 0;
        let namespace = Namespace::Flat;
        let entry = create_package_entry("package");

        let path = location_from_root(chunk_size, &namespace, entry.name());

        // Should just produce: package
        assert_eq!(path, Path::new("package"));
    }
}
