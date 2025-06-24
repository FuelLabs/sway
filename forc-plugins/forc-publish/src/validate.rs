use crate::error::{Error, Result};
use forc_pkg::manifest::{GenericManifestFile, ManifestFile};
use std::path::Path;

/// Checks the following cases for an early error generation:
///  1. Target dir doesn't contain a Forc.toml
///  2. Target manifest file doesn't contain a version
///  3. Target project's dependencies are not all version based (git or path
///     based dep detection)
pub fn validate_dir(path: &Path) -> Result<()> {
    // Check if Forc.toml exists
    let forc_toml_path = path.join("Forc.toml");
    if !forc_toml_path.exists() {
        return Err(Error::ForcTomlNotFound);
    }

    let manifest_file = ManifestFile::from_file(forc_toml_path)?;

    match manifest_file {
        ManifestFile::Package(package_manifest_file) => {
            // Check if the version exists
            if package_manifest_file.as_ref().project.version.is_none() {
                return Err(Error::MissingVersionField);
            }

            // Check if all the dependencies are declared with dep
            for (dep_name, dep) in package_manifest_file
                .as_ref()
                .dependencies
                .iter()
                .flat_map(|deps| deps.iter())
            {
                if dep.version().is_none() {
                    return Err(Error::DependencyMissingVersion(dep_name.to_string()));
                }
            }
        }
        ManifestFile::Workspace(_) => {
            return Err(Error::WorkspaceNotSupported);
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::validate_dir;
    use std::path::PathBuf;

    #[test]
    fn without_version_should_fail() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let tests_path = manifest_dir.join("tests").join("data");

        let no_version_project_test = tests_path.join("without_version");
        let res = validate_dir(&no_version_project_test);

        assert!(res.is_err());
        assert!(res.err().is_some_and(|err| {
            err.to_string() == "Project is missing a version field, add one under [project]"
        }));
    }

    #[test]
    fn deps_without_version_should_fail() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let tests_path = manifest_dir.join("tests").join("data");

        let no_version_project_test = tests_path.join("deps_without_version");
        let res = validate_dir(&no_version_project_test);

        assert!(res.is_err());
        assert!(res.err().is_some_and(|err| {
            err.to_string() == "dep_a is not a forc.pub dependency, depend on it using version."
        }));
    }

    #[test]
    fn success_without_deps() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let tests_path = manifest_dir.join("tests").join("data");

        let success_without_deps = tests_path.join("success_with_no_deps");
        validate_dir(&success_without_deps).unwrap()
    }

    #[test]
    fn success_with_deps() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let tests_path = manifest_dir.join("tests").join("data");

        let success_without_deps = tests_path.join("success_with_deps");
        validate_dir(&success_without_deps).unwrap()
    }
}
