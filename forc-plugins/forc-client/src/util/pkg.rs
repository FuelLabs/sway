use anyhow::Result;
use forc_pkg::manifest::GenericManifestFile;
use forc_pkg::{self as pkg, manifest::ManifestFile, BuildOpts, BuildPlan};
use forc_util::user_forc_directory;
use pkg::{build_with_options, BuiltPackage, PackageManifestFile};
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::{collections::HashMap, path::Path, sync::Arc};
use sway_utils::{MAIN_ENTRY, MANIFEST_FILE_NAME, SRC_DIR};

/// The name of the folder that forc generated proxy contract project will reside at.
pub const PROXY_CONTRACT_FOLDER_NAME: &str = ".generated_proxy_contracts";
/// Forc.toml for the default proxy contract that 'generate_proxy_contract_src()' returns.
pub const PROXY_CONTRACT_FORC_TOML: &str = r#"
[project]
authors = ["Fuel Labs <contact@fuel.sh>"]
entry = "main.sw"
license = "Apache-2.0"
name = "src14_owned_proxy"

[dependencies]
standards = { git = "https://github.com/FuelLabs/sway-standards", tag = "v0.5.1" }
sway_libs = { git = "https://github.com/FuelLabs/sway-libs", tag = "v0.23.0" }
"#;

/// Generates source code for proxy contract owner set to the given 'addr'.
pub(crate) fn generate_proxy_contract_src(addr: &str, impl_contract_id: &str) -> String {
    format!(
        r#"
contract;
use ::sway_libs::{{
    ownership::errors::InitializationError,
    upgradability::{{
        _proxy_owner,
        _proxy_target,
        _set_proxy_owner,
        _set_proxy_target,
        only_proxy_owner,
    }},
}};
use standards::{{src14::{{SRC14, SRC14Extension}}, src5::State}};
use std::execution::run_external;

abi OwnedProxy {{
    #[storage(write)]
    fn initialize_proxy();

    #[storage(write)]
    fn set_proxy_owner(new_proxy_owner: State);
}}

configurable {{
    INITIAL_TARGET: Option<ContractId> = Some(ContractId::from({impl_contract_id})),
    INITIAL_OWNER: State = State::Initialized(Identity::Address(Address::from({addr}))),
}}

#[namespace(SRC14)]
storage {{
    target: Option<ContractId> = None,
    proxy_owner: State = State::Uninitialized,
}}

impl SRC14 for Contract {{
    #[storage(read, write)]
    fn set_proxy_target(new_target: ContractId) {{
        only_proxy_owner(storage.proxy_owner);
        _set_proxy_target(new_target);
    }}

    #[storage(read)]
    fn proxy_target() -> Option<ContractId> {{
        _proxy_target()
    }}
}}

impl SRC14Extension for Contract {{
    #[storage(read)]
    fn proxy_owner() -> State {{
        _proxy_owner(storage.proxy_owner)
    }}
}}

impl OwnedProxy for Contract {{
    #[storage(write)]
    fn initialize_proxy() {{
        require(
            _proxy_owner(storage.proxy_owner) == State::Uninitialized,
            InitializationError::CannotReinitialized,
        );

        storage.target.write(INITIAL_TARGET);
        storage.proxy_owner.write(INITIAL_OWNER);
    }}

    #[storage(write)]
    fn set_proxy_owner(new_proxy_owner: State) {{
        _set_proxy_owner(new_proxy_owner, storage.proxy_owner);
    }}
}}

#[fallback]
#[storage(read)]
fn fallback() {{
    run_external(_proxy_target().expect("FallbackError::TargetNotSet"))
}}
        "#
    )
}

/// Updates the given package manifest file such that the address field under the proxy table updated to the given value.
/// Updated manifest file is written back to the same location, without thouching anything else such as comments etc.
/// A safety check is done to ensure the proxy table exists before attempting to udpdate the value.
pub(crate) fn update_proxy_address_in_manifest(
    address: &str,
    manifest: &PackageManifestFile,
) -> Result<()> {
    let mut toml = String::new();
    let mut file = File::open(manifest.path())?;
    file.read_to_string(&mut toml)?;
    let mut manifest_toml = toml.parse::<toml_edit::Document>()?;
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

/// Creates a proxy contract project at the given path, adds a forc.toml and source file.
pub(crate) fn create_proxy_contract(
    owner_addr: &fuels_core::types::Address,
    impl_contract_id: &fuel_tx::ContractId,
    pkg_name: &str,
) -> Result<PathBuf> {
    let owner_addr = &format!("0x{}", owner_addr);
    let impl_contract_id = &format!("0x{}", impl_contract_id);

    // Create the proxy contract folder.
    let proxy_contract_dir = user_forc_directory()
        .join(PROXY_CONTRACT_FOLDER_NAME)
        .join(pkg_name);
    std::fs::create_dir_all(&proxy_contract_dir)?;

    // Create the Forc.toml
    let mut f = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(proxy_contract_dir.join(MANIFEST_FILE_NAME))?;
    write!(f, "{}", PROXY_CONTRACT_FORC_TOML)?;

    // Create the src folder
    std::fs::create_dir_all(proxy_contract_dir.join(SRC_DIR))?;

    // Create main.sw
    let mut f = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(proxy_contract_dir.join(SRC_DIR).join(MAIN_ENTRY))?;

    let contract_str = generate_proxy_contract_src(owner_addr, impl_contract_id);
    write!(f, "{}", contract_str)?;
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

/// Build a proxy contract owned by the deployer.
/// First creates the contract project at the current dir. The source code for the proxy contract is updated
/// with 'owner_addr'.
pub fn build_proxy_contract(
    owner_addr: &fuels_core::types::Address,
    impl_contract_id: &fuel_tx::ContractId,
    pkg_name: &str,
    build_opts: &BuildOpts,
) -> Result<Arc<BuiltPackage>> {
    let proxy_contract_dir = create_proxy_contract(owner_addr, impl_contract_id, pkg_name)?;
    let mut build_opts = build_opts.clone();
    let proxy_contract_dir_str = format!("{}", proxy_contract_dir.clone().display());
    build_opts.pkg.path = Some(proxy_contract_dir_str);
    let built_pkgs = built_pkgs(&proxy_contract_dir, &build_opts)?;
    let built_pkg = built_pkgs
        .first()
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("could not get proxy contract"))?;
    Ok(built_pkg)
}
