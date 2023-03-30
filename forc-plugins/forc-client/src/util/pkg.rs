use std::{collections::HashMap, path::Path, sync::Arc};

use anyhow::{bail, Result};
use forc_pkg::{self as pkg, manifest::ManifestFile, BuildOpts, BuildPlan};
use pkg::{build_with_options, BuiltPackage, PackageManifestFile};

#[derive(Clone, Debug)]
pub struct BuiltPackageWithManifest(Arc<BuiltPackage>, PackageManifestFile);

impl BuiltPackageWithManifest {
    /// Returns an immutable reference into the Arc<BuiltPackage>.
    pub fn built_package(&self) -> &Arc<BuiltPackage> {
        &self.0
    }

    /// Returns an immutable reference into the PackageManifestFile.
    pub fn package_manifest_file(&self) -> &PackageManifestFile {
        &self.1
    }
}

pub(crate) fn built_pkgs_with_manifest(
    path: &Path,
    build_opts: BuildOpts,
) -> Result<Vec<BuiltPackageWithManifest>> {
    let manifest_file = ManifestFile::from_dir(path)?;
    let mut member_manifests = manifest_file.member_manifests()?;
    let lock_path = manifest_file.lock_path()?;
    let build_plan = BuildPlan::from_lock_and_manifests(
        &lock_path,
        &member_manifests,
        build_opts.pkg.locked,
        build_opts.pkg.offline,
    )?;
    let graph = build_plan.graph();
    let built = build_with_options(build_opts)?;
    let mut built_pkgs: HashMap<&pkg::Pinned, Arc<_>> = built.into_members().collect();
    let mut pkgs_with_manifest = Vec::new();
    for member_index in build_plan.member_nodes() {
        let pkg = &graph[member_index];
        let pkg_name = &pkg.name;
        // Check if the current member is built.
        //
        // For individual members of the workspace, member nodes would be iterating
        // over all the members but only the relevant member would be built.
        if let Some(built_pkg) = built_pkgs.remove(pkg) {
            let member_manifest = member_manifests
                .remove(pkg_name)
                .expect("Member manifest file is missing");
            pkgs_with_manifest.push(BuiltPackageWithManifest(built_pkg, member_manifest));
        }
    }

    if pkgs_with_manifest.is_empty() {
        bail!("No built packages collected");
    }

    Ok(pkgs_with_manifest)
}
