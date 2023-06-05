use std::{collections::HashMap, path::Path, sync::Arc};

use anyhow::Result;
use forc_pkg::{self as pkg, manifest::ManifestFile, BuildOpts, BuildPlan};
use pkg::{build_with_options, BuiltPackage};

pub(crate) fn built_pkgs(path: &Path, build_opts: BuildOpts) -> Result<Vec<Arc<BuiltPackage>>> {
    let manifest_file = ManifestFile::from_dir(path)?;
    let lock_path = manifest_file.lock_path()?;
    let build_plan = BuildPlan::from_lock_and_manifests(
        &lock_path,
        &manifest_file.member_manifests()?,
        build_opts.pkg.locked,
        build_opts.pkg.offline,
    )?;
    let graph = build_plan.graph();
    let built = build_with_options(build_opts)?;
    let mut members: HashMap<&pkg::Pinned, Arc<_>> = built.into_members().collect();
    let mut built_pkgs = Vec::new();

    for member_index in build_plan.member_nodes() {
        let pkg = &graph[member_index];
        // Check if the current member is built.
        //
        // For individual members of the workspace, member nodes would be iterating
        // over all the members but only the relevant member would be built.
        if let Some(built_pkg) = members.remove(pkg) {
            built_pkgs.push(built_pkg);
        }
    }

    Ok(built_pkgs)
}
