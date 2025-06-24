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
