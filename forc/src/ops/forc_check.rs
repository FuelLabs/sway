use crate::utils::SWAY_GIT_TAG;
use anyhow::Result;
use forc_pkg::{self as pkg, ManifestFile};
use forc_util::lock_path;
use std::path::Path;

pub fn check(manifest_dir: &Path) -> Result<sway_core::CompileAstResult> {
    let manifest = ManifestFile::from_dir(manifest_dir, SWAY_GIT_TAG)?;

    let config = &pkg::BuildConfig {
        print_ir: false,
        print_finalized_asm: false,
        print_intermediate_asm: false,
        silent: true,
    };

    let lock_path = lock_path(manifest.dir());

    let build_plan = pkg::BuildPlan::from_lock_file(&lock_path, SWAY_GIT_TAG)?;

    // Build it!
    pkg::check(&build_plan, config, SWAY_GIT_TAG)
}
