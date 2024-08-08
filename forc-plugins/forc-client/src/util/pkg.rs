use anyhow::{bail, Result};
use forc_pkg::manifest::GenericManifestFile;
use forc_pkg::{self as pkg, manifest::ManifestFile, BuildOpts, BuildPlan};
use forc_util::user_forc_directory;
use fuel_abi_types::abi::program::ProgramABI;
use fuel_crypto::SecretKey;
use fuel_tx::{ContractId, Salt, StorageSlot};
use fuels::types::transaction::TxPolicies;
use fuels_accounts::provider::Provider;
use fuels_accounts::wallet::WalletUnlocked;
use pkg::{build_with_options, BuiltPackage, PackageManifestFile};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::{collections::HashMap, path::Path, sync::Arc};
use sway_utils::{MAIN_ENTRY, MANIFEST_FILE_NAME, SRC_DIR};

/// The name of the folder that forc generated proxy contract project will reside at.
pub const GENERATED_CONTRACT_FOLDER_NAME: &str = ".generated_contracts";
pub const PROXY_CONTRACT_BIN: &[u8] = include_bytes!("../../proxy_abi/proxy_contract.bin");
pub const PROXY_CONTRACT_STORAGE_SLOTS: &str =
    include_str!("../../proxy_abi/proxy_contract-storage_slots.json");
pub const PROXY_BIN_FILE_NAME: &str = "proxy.bin";
pub const PROXY_STORAGE_SLOTS_FILE_NAME: &str = "proxy-storage_slots.json";
pub const LOADER_CONTRACT_FORC_TOML: &str = r#"
[project]
authors = ["Fuel Labs <contact@fuel.sh>"]
entry = "main.sw"
license = "Apache-2.0"
name = "proxy_contract"

[dependencies]
"#;

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
pub(crate) fn create_proxy_contract(pkg_name: &str) -> Result<PathBuf> {
    // Create the proxy contract folder.
    let proxy_contract_dir = user_forc_directory()
        .join(GENERATED_CONTRACT_FOLDER_NAME)
        .join(format!("{}-proxy", pkg_name));
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

pub fn generate_proxy_contract_with_chunking_src(
    abi: &ProgramABI,
    num_chunks: usize,
    chunk_contract_ids: &[String],
) -> String {
    let mut contract = String::new();
    let types = create_type_map(abi);

    // Contract header
    contract.push_str("contract;\n\n");

    // Configurables
    contract.push_str("configurable {\n");
    for i in 1..=num_chunks {
        contract.push_str(&format!(
            "    TARGET_{}: ContractId = ContractId::from({}),\n",
            i,
            chunk_contract_ids[i - 1]
        ));
    }
    contract.push_str("}\n\n");

    // ABI
    contract.push_str("abi LoaderContract {\n");
    for function in &abi.functions {
        let inputs = function
            .inputs
            .iter()
            .map(|input| {
                format!(
                    "{}: {}",
                    input.name,
                    type_id_to_string(input.type_id, &types)
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        let output = type_id_to_string(function.output.type_id, &types);
        contract.push_str(&format!(
            "    fn {}({}) -> {};\n",
            function.name, inputs, output
        ));
    }
    contract.push_str("}\n\n");

    // Implementation
    contract.push_str("impl LoaderContract for Contract {\n");
    for function in &abi.functions {
        let inputs = function
            .inputs
            .iter()
            .map(|input| {
                format!(
                    "{}: {}",
                    input.name,
                    type_id_to_string(input.type_id, &types)
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        let output = type_id_to_string(function.output.type_id, &types);
        contract.push_str(&format!(
            "    fn {}({}) -> {} {{\n",
            function.name, inputs, output
        ));
        contract.push_str(&format!("        run_external{}(", num_chunks));
        for i in 1..=num_chunks {
            if i > 1 {
                contract.push_str(", ");
            }
            contract.push_str(&format!("TARGET_{}", i));
        }
        contract.push_str(")\n    }\n");
    }
    contract.push_str("}\n\n");

    // run_external function
    contract.push_str(&generate_run_external(num_chunks));

    contract
}

/// Creates a map from type id to the name of the type, so that we can do ,ook
fn create_type_map(abi: &ProgramABI) -> HashMap<usize, String> {
    abi.types
        .iter()
        .map(|t| (t.type_id, t.type_field.clone()))
        .collect()
}

fn type_id_to_string(type_id: usize, types: &HashMap<usize, String>) -> String {
    types
        .get(&type_id)
        .cloned()
        .unwrap_or_else(|| format!("Type{}", type_id))
}

/// Generates an run external function given the `num_targets`. This function
/// is later used in the generated loader contract.
fn generate_run_external(num_targets: usize) -> String {
    let mut func = String::new();

    func.push_str(&format!("fn run_external{}(", num_targets));
    for i in 1..=num_targets {
        if i > 1 {
            func.push_str(", ");
        }
        func.push_str(&format!("load_target{}: ContractId", i));
    }
    func.push_str(") -> ! {\n");

    // Generate assembly
    func.push_str("    asm(\n");
    for i in 1..=num_targets {
        func.push_str(&format!("        load_target{}: load_target{},\n", i, i));
    }
    for i in 2..=num_targets {
        func.push_str(&format!("        load_target{}_heap,\n", i));
    }
    func.push_str("        heap_alloc_size,\n");
    for i in 1..=num_targets {
        func.push_str(&format!("        length{},\n", i));
    }
    func.push_str("        ssp_saved,\n");
    func.push_str("        cur_stack_size,\n");
    func.push_str("    ) {\n");

    // Get lengths of all chunks
    for i in 1..=num_targets {
        func.push_str(&format!("        csiz length{} load_target{};\n", i, i));
    }

    // Store load_target2 and onwards on the heap
    for i in 2..=num_targets {
        func.push_str("        addi heap_alloc_size zero i32;\n");
        func.push_str("        aloc heap_alloc_size;\n");
        func.push_str(&format!(
            "        mcp hp load_target{} heap_alloc_size;\n",
            i
        ));
        func.push_str(&format!("        move load_target{}_heap hp;\n", i));
    }

    // Save the old $ssp value and shrink the stack
    func.push_str("        move ssp_saved ssp;\n");
    func.push_str("        sub cur_stack_size sp ssp;\n");
    func.push_str("        cfs cur_stack_size;\n");

    // Do the loads
    func.push_str("        ldc load_target1 zero length1;\n");
    for i in 2..=num_targets {
        func.push_str(&format!(
            "        ldc load_target{}_heap zero length{};\n",
            i, i
        ));
    }

    // Set up jump
    func.push_str("        addi heap_alloc_size zero i64;\n");
    func.push_str("        aloc heap_alloc_size;\n");
    func.push_str("        sw hp ssp_saved i0;\n");
    func.push_str("    }\n");
    func.push_str("    __jmp_mem()\n");
    func.push_str("}\n");

    func
}

/// Creates a loader contract project, ready to be built at the given path.
///
/// Constructs:
/// - The project folder with a correct structure if it is not already there.
/// - Loader contract soruce code, that is created specifcally for number of
///   chunks in hand.
pub(crate) fn create_chunk_loader_contract(
    abi: &ProgramABI,
    chunk_contract_ids: &[String],
    num_chunks: usize,
    pkg_name: &str,
) -> Result<PathBuf> {
    // Create the proxy contract folder.
    let proxy_contract_dir = user_forc_directory()
        .join(GENERATED_CONTRACT_FOLDER_NAME)
        .join(pkg_name);
    std::fs::create_dir_all(&proxy_contract_dir)?;

    // Create the Forc.toml
    let mut f = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(proxy_contract_dir.join(MANIFEST_FILE_NAME))?;
    write!(f, "{}", LOADER_CONTRACT_FORC_TOML)?;

    // Create the src folder
    std::fs::create_dir_all(proxy_contract_dir.join(SRC_DIR))?;

    // Create main.sw
    let mut f = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(proxy_contract_dir.join(SRC_DIR).join(MAIN_ENTRY))?;

    let contract_str =
        generate_proxy_contract_with_chunking_src(abi, num_chunks, chunk_contract_ids);
    write!(f, "{}", contract_str)?;
    Ok(proxy_contract_dir)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContractChunk {
    id: usize,
    size: usize,
    bytecode: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeployedContractChunk {
    contract_id: ContractId,
}

impl DeployedContractChunk {
    pub fn contract_id(&self) -> &ContractId {
        &self.contract_id
    }
}

impl ContractChunk {
    pub fn new(id: usize, size: usize, bytecode: Vec<u8>) -> Self {
        Self { id, size, bytecode }
    }

    pub async fn deploy(
        self,
        provider: &Provider,
        salt: &Salt,
        signing_key: &SecretKey,
    ) -> anyhow::Result<DeployedContractChunk> {
        let wallet = WalletUnlocked::new_from_private_key(*signing_key, Some(provider.clone()));

        let contract_chunk_storage_slot = StorageSlot::default();
        let contract_chunk = fuels::programs::contract::Contract::new(
            self.bytecode,
            *salt,
            vec![contract_chunk_storage_slot],
        );

        let policies = TxPolicies::default();
        let bech32 = contract_chunk.deploy(&wallet, policies).await?;
        let contract_id = ContractId::from(bech32);
        Ok(DeployedContractChunk { contract_id })
    }
}

/// Split bytecode into chunks of a specified maximum size. Meaning that each
/// chunk up until the last one, is guaranteed to be `chunk_size`, and
/// `chunk_size` is guaranteed to be divisble by 8, and will result an error
/// otherwise. This requirement comes from VM, as LDC'ed bytecode is appended
/// to word boundary.
pub fn split_into_chunks(bytecode: Vec<u8>, chunk_size: usize) -> Result<Vec<ContractChunk>> {
    // This is done so that LDC'ed bytecode aligns perfectly, as the VM appends
    // them to word boundary. This should normally be the case if the bytecode
    // is not modified manually.
    assert!(chunk_size % 8 == 0);
    if chunk_size % 8 != 0 {
        bail!(
            "Chunks size is not divisible by 8, chunk size: {}",
            chunk_size
        );
    }
    let mut chunks = Vec::new();

    for (id, chunk) in bytecode.chunks(chunk_size).enumerate() {
        let chunk = chunk.to_vec();
        let size = chunk.len();
        let contract_chunk = ContractChunk::new(id, size, chunk);
        chunks.push(contract_chunk);
    }

    Ok(chunks)
}
