use crate::{
    cmd::{ForcNodeCmd, Mode},
    consts::MIN_FUEL_CORE_VERSION,
    util::get_fuel_core_version,
};
use forc_util::forc_result_bail;
use semver::Version;
use std::process::Child;

/// First checks locally installed `forc-node` version and compares it with
/// `consts::MIN_FUEL_CORE_VERSION`. If local version is acceptable, proceeding
/// with the correct mode of operation.
pub async fn run(cmd: ForcNodeCmd) -> anyhow::Result<Option<Child>> {
    let current_version = get_fuel_core_version()?;
    let supported_min_version = Version::parse(MIN_FUEL_CORE_VERSION)?;
    if current_version < supported_min_version {
        forc_result_bail!(format!(
            "Minimum supported fuel core version is {MIN_FUEL_CORE_VERSION}, system version: {}",
            current_version
        ));
    }
    let forc_node_handle = match cmd.mode {
        Mode::Local(local) => crate::local::op::run(local, cmd.dry_run).await?,
        Mode::Testnet(testnet) => crate::testnet::op::run(testnet, cmd.dry_run).await?,
        Mode::Ignition(ignition) => crate::ignition::op::run(ignition, cmd.dry_run).await?,
    };
    Ok(forc_node_handle)
}
