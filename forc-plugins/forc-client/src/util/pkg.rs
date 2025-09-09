use anyhow::Result;
use forc_pkg::manifest::GenericManifestFile;
use forc_pkg::{self as pkg, manifest::ManifestFile, BuildOpts, BuildPlan};
use forc_util::user_forc_directory;
use pkg::{build_with_options, BuiltPackage, PackageManifestFile};
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::{collections::HashMap, path::Path, sync::Arc};
use fuels::types::chain_info::ChainInfo;
use crate::util::target::Target;
use std::str::FromStr;

/// The name of the folder that forc generated proxy contract project will reside at.
pub const GENERATED_CONTRACT_FOLDER_NAME: &str = ".generated_contracts";
pub const PROXY_CONTRACT_BIN: &[u8] = include_bytes!("../../proxy_abi/proxy_contract.bin");
pub const PROXY_CONTRACT_STORAGE_SLOTS: &str =
    include_str!("../../proxy_abi/proxy_contract-storage_slots.json");
pub const PROXY_BIN_FILE_NAME: &str = "proxy.bin";
pub const PROXY_STORAGE_SLOTS_FILE_NAME: &str = "proxy-storage_slots.json";

/// Updates the given package manifest file such that the address field under the proxy table updated to the given value.
/// Updated manifest file is written back to the same location, without thouching anything else such as comments etc.
/// A safety check is done to ensure the proxy table exists before attempting to update the value.
pub(crate) fn update_proxy_address_in_manifest(
    address: &str,
    manifest: &PackageManifestFile,
) -> Result<()> {
    let mut toml = String::new();
    let mut file = File::open(manifest.path())?;
    file.read_to_string(&mut toml)?;
    let mut manifest_toml = toml.parse::<toml_edit::DocumentMut>()?;
    if manifest.proxy().is_some() {
        manifest_toml["proxy"]["address"] = toml_edit::value(address);
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(manifest.path())?;
        file.write_all(manifest_toml.to_string().as_bytes())?;
    }
    Ok(())
}

/// Updates the given package manifest file with network-specific proxy address.
/// This function determines the network from the chain info and updates the appropriate 
/// network entry in the proxy.addresses table.
pub(crate) fn update_proxy_address_in_manifest_for_network(
    address: &str,
    manifest: &PackageManifestFile,
    chain_info: &ChainInfo,
) -> Result<()> {
    let mut toml = String::new();
    let mut file = File::open(manifest.path())?;
    file.read_to_string(&mut toml)?;
    let mut manifest_toml = toml.parse::<toml_edit::DocumentMut>()?;
    
    if manifest.proxy().is_some() {
        // Determine network name from chain info
        let target = Target::from_str(&chain_info.name).unwrap_or_default();
        let network_name = match target {
            Target::Testnet => "testnet",
            Target::Mainnet => "mainnet", 
            Target::Devnet => "devnet",
            Target::Local => "local",
        };

        // Check if we're using the new addresses format or need to convert
        if manifest.proxy().unwrap().addresses.is_some() {
            // Update network-specific address in addresses table
            if !manifest_toml["proxy"]["addresses"].is_table() {
                manifest_toml["proxy"]["addresses"] = toml_edit::table();
            }
            manifest_toml["proxy"]["addresses"][network_name] = toml_edit::value(address);
        } else if manifest.proxy().unwrap().address.is_some() {
            // Migration from single address to network-specific addresses
            // Move current address to network-specific and add new one
            
            // Remove the old single address field
            if manifest_toml["proxy"]["address"].is_value() {
                manifest_toml["proxy"]["address"] = toml_edit::Item::None;
            }
            
            // Create addresses table and add the new address for this network
            manifest_toml["proxy"]["addresses"] = toml_edit::table();
            manifest_toml["proxy"]["addresses"][network_name] = toml_edit::value(address);
            
            // Optionally preserve the old address under a generic network name
            // This is commented out to avoid confusion, but could be enabled if desired
            // manifest_toml["proxy"]["addresses"]["previous"] = toml_edit::value(current_address);
        } else {
            // No address configured yet, create addresses table with this network
            manifest_toml["proxy"]["addresses"] = toml_edit::table();
            manifest_toml["proxy"]["addresses"][network_name] = toml_edit::value(address);
        }

        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(manifest.path())?;
        file.write_all(manifest_toml.to_string().as_bytes())?;
    }
    
    Ok(())
}

/// Creates a proxy contract project at the given path, adds a forc.toml and source file.
pub(crate) fn create_proxy_contract(pkg_name: &str) -> Result<PathBuf> {
    // Create the proxy contract folder.
    let proxy_contract_dir = user_forc_directory()
        .join(GENERATED_CONTRACT_FOLDER_NAME)
        .join(format!("{pkg_name}-proxy"));
    std::fs::create_dir_all(&proxy_contract_dir)?;
    std::fs::write(
        proxy_contract_dir.join(PROXY_BIN_FILE_NAME),
        PROXY_CONTRACT_BIN,
    )?;
    std::fs::write(
        proxy_contract_dir.join(PROXY_STORAGE_SLOTS_FILE_NAME),
        PROXY_CONTRACT_STORAGE_SLOTS,
    )?;

    Ok(proxy_contract_dir)
}

pub(crate) fn built_pkgs(path: &Path, build_opts: &BuildOpts) -> Result<Vec<Arc<BuiltPackage>>> {
    let manifest_file = ManifestFile::from_dir(path)?;
    let lock_path = manifest_file.lock_path()?;
    let build_plan = BuildPlan::from_lock_and_manifests(
        &lock_path,
        &manifest_file.member_manifests()?,
        build_opts.pkg.locked,
        build_opts.pkg.offline,
        &build_opts.pkg.ipfs_node,
    )?;
    let graph = build_plan.graph();
    let built = build_with_options(build_opts, None)?;
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
