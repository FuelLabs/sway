use std::path::PathBuf;

use crate::cli::{AddCommand, RemoveCommand};
use anyhow::{bail, Result};
use forc_pkg::manifest::{Dependency, DependencyDetails, ManifestFile};

pub fn add(
    /*the command from the user that is typed into the terminal*/ command: AddCommand,
) -> Result<()> {
    // 1. How will forc know where to add a dependency to?

    // type of the variable
    let AddCommand {
        dependency,    // whatever library the user is trying to add to the toml
        manifest_path, // the path to forc.toml
    } = command; // variable that we got from the add function

    // matches the path to the forc.toml
    let dir = match manifest_path {
        Some(ref path) => PathBuf::from(path), // if there is _some_ manifest_path, then store the manifest_path
        None => std::env::current_dir()?,      // if not, then use the current directory
    };
    let manifest = ManifestFile::from_dir(&dir)?; //stores the forc.toml in a varible, and checks if it is a package or a workspace
    let pkg_manifest = if let ManifestFile::Package(pkg_manifest) = &manifest {
        // if the manifest is a package...
        pkg_manifest // store this package
    } else {
        bail!("forc-edit does not support workspaces.") // otherwise bail
    };

    let dependency_path = PathBuf::from(&dependency); // create a path buffer from the dependency string
    let dep_pkg = if let Ok(dep_path) = ManifestFile::from_dir(&dependency_path) {
        // if the dependency path exists...
        if let ManifestFile::Package(dep_pkg) = dep_path {
            // ...store this variable
            dep_pkg
        } else {
            bail!("forc-edit does not support workspaces.") // otherwise bail
        }
    } else {
        bail!("dependency path does not contain a forc.toml") // otherwise bail
    };
    let key = dep_pkg.project.name.clone(); // the key is the name of the project
    let value = Dependency::Detailed(DependencyDetails {
        // the value is the path to the dependency
        version: None,
        path: Some(dependency),
        git: None,
        branch: None,
        tag: None,
        package: None,
        rev: None,
    });

    // 2. How will we find the dependency block in the forc.toml?
    //
    // Todo: can I insert a new key value pair directly into this BTreeMap?
    if let Some(mut deps) = pkg_manifest.dependencies.clone() {
        // 3. Write new dependency to that block (name and path)
        deps.insert(key, value);
    }

    Ok(())
}
pub fn remove(_command: RemoveCommand) -> Result<()> {
    Ok(())
}
