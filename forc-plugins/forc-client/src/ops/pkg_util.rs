use std::{collections::HashMap, path::Path};

use anyhow::Result;
use forc_pkg::{self as pkg, manifest::ManifestFile, BuildOpts, BuildPlan};
use pkg::{manifest::MemberManifestFiles, BuiltPackage, PackageManifestFile};

fn built_pkgs(
    build_opts: BuildOpts,
    member_manifests: &MemberManifestFiles,
) -> Result<HashMap<String, BuiltPackage>> {
    let built = forc_pkg::build_with_options(build_opts)?;
    match built {
        pkg::Built::Package(built_pkg) => {
            let pkg_name = member_manifests
                .keys()
                .next()
                .expect("built package is missing");
            Ok(std::iter::once((pkg_name.clone(), *built_pkg)).collect())
        }
        pkg::Built::Workspace(built_workspace) => Ok(built_workspace),
    }
}

pub(crate) fn built_pkgs_with_manifest(
    path: &Path,
    build_opts: BuildOpts,
) -> Result<Vec<(PackageManifestFile, BuiltPackage)>> {
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
    let mut built_pkgs = built_pkgs(build_opts, &member_manifests)?;
    let pkgs_with_manifest: Vec<(PackageManifestFile, BuiltPackage)> = build_plan
        .member_nodes()
        .map(|member_index| {
            let pkg_name = &graph[member_index].name;
            let member_manifest = member_manifests
                .remove(pkg_name)
                .expect("Member manifest file is missing");
            let built_pkg = built_pkgs
                .remove(pkg_name)
                .expect("Built package is missing");
            (member_manifest, built_pkg)
        })
        .collect();

    Ok(pkgs_with_manifest)
}
