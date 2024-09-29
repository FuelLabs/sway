use crate::{
    cmd,
    constants::TX_SUBMIT_TIMEOUT_MS,
    util::{
        account::ForcClientAccount,
        node_url::get_node_url,
        pkg::{built_pkgs, create_proxy_contract, update_proxy_address_in_manifest},
        target::Target,
        tx::{
            prompt_forc_wallet_password, select_account, update_proxy_contract_target,
            SignerSelectionMode,
        },
    },
};
use anyhow::{bail, Context, Result};
use forc_pkg::manifest::GenericManifestFile;
use forc_pkg::{self as pkg, PackageManifestFile};
use forc_tracing::{println_action_green, println_warning};
use forc_util::default_output_directory;
use forc_wallet::utils::default_wallet_path;
use fuel_core_client::client::types::{ChainInfo, TransactionStatus};
use fuel_core_client::client::FuelClient;
use fuel_crypto::fuel_types::ChainId;
use fuel_tx::{Salt, Transaction};
use fuel_vm::prelude::*;
use fuels::{
    macros::abigen,
    programs::contract::{LoadConfiguration, StorageConfiguration},
    types::{bech32::Bech32ContractId, transaction_builders::Blob},
};
use fuels_accounts::{provider::Provider, Account, ViewOnlyAccount};
use fuels_core::types::{transaction::TxPolicies, transaction_builders::CreateTransactionBuilder};
use futures::FutureExt;
use pkg::{manifest::build_profile::ExperimentalFlags, BuildProfile, BuiltPackage};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
    time::Duration,
};
use sway_core::language::parsed::TreeType;
use sway_core::BuildTarget;

/// Maximum contract size allowed for a single contract. If the target
/// contract size is bigger than this amount, forc-deploy will automatically
/// starts dividing the contract and deploy them in chunks automatically.
/// The value is in bytes.
const MAX_CONTRACT_SIZE: usize = 100_000;

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub struct DeployedContract {
    pub id: fuel_tx::ContractId,
    pub proxy: Option<fuel_tx::ContractId>,
    pub chunked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentArtifact {
    transaction_id: String,
    salt: String,
    network_endpoint: String,
    chain_id: ChainId,
    contract_id: String,
    deployment_size: usize,
    deployed_block_height: Option<u32>,
}

impl DeploymentArtifact {
    pub fn to_file(
        &self,
        output_dir: &Path,
        pkg_name: &str,
        contract_id: ContractId,
    ) -> Result<()> {
        if !output_dir.exists() {
            std::fs::create_dir_all(output_dir)?;
        }

        let deployment_artifact_json = format!("{pkg_name}-deployment-0x{contract_id}");
        let deployments_path = output_dir
            .join(deployment_artifact_json)
            .with_extension("json");
        let deployments_file = std::fs::File::create(deployments_path)?;
        serde_json::to_writer_pretty(&deployments_file, &self)?;
        Ok(())
    }
}

type ContractSaltMap = BTreeMap<String, Salt>;

/// Takes the contract member salt inputs passed via the --salt option, validates them against
/// the manifests and returns a ContractSaltMap (BTreeMap of contract names to salts).
fn validate_and_parse_salts<'a>(
    salt_args: &[String],
    manifests: impl Iterator<Item = &'a PackageManifestFile>,
) -> Result<ContractSaltMap> {
    let mut contract_salt_map = BTreeMap::default();

    // Parse all the salt arguments first, and exit if there are errors in this step.
    for salt_arg in salt_args {
        if let Some((given_contract_name, salt)) = salt_arg.split_once(':') {
            let salt = salt
                .parse::<Salt>()
                .map_err(|e| anyhow::anyhow!(e))
                .unwrap();

            if let Some(old) = contract_salt_map.insert(given_contract_name.to_string(), salt) {
                bail!("2 salts provided for contract '{given_contract_name}':\n  {old}\n  {salt}");
            };
        } else {
            bail!("Invalid salt provided - salt must be in the form <CONTRACT_NAME>:<SALT> when deploying a workspace");
        }
    }

    for manifest in manifests {
        for (dep_name, contract_dep) in manifest.contract_deps() {
            let dep_pkg_name = contract_dep.dependency.package().unwrap_or(dep_name);
            if let Some(declared_salt) = contract_salt_map.get(dep_pkg_name) {
                bail!(
                    "Redeclaration of salt using the option '--salt' while a salt exists for contract '{}' \
                    under the contract dependencies of the Forc.toml manifest for '{}'\n\
                    Existing salt: '0x{}',\nYou declared: '0x{}'\n",
                    dep_pkg_name,
                    manifest.project_name(),
                    contract_dep.salt,
                    declared_salt,
                    );
            }
        }
    }

    Ok(contract_salt_map)
}

/// Depending on the cli options user passed, either returns storage slots from
/// compiled package, or the ones user provided as overrides.
fn resolve_storage_slots(
    command: &cmd::Deploy,
    compiled: &BuiltPackage,
) -> Result<Vec<fuel_tx::StorageSlot>> {
    let mut storage_slots =
        if let Some(storage_slot_override_file) = &command.override_storage_slots {
            let storage_slots_file = std::fs::read_to_string(storage_slot_override_file)?;
            let storage_slots: Vec<StorageSlot> = serde_json::from_str(&storage_slots_file)?;
            storage_slots
        } else {
            compiled.storage_slots.clone()
        };
    storage_slots.sort();
    Ok(storage_slots)
}

/// Creates blobs from the contract to deploy contracts that are larger than
/// `MAX_CONTRACT_SIZE`. Created blobs are deployed, and a loader contract is
/// generated such that it loads all the deployed blobs, and provides the user
/// a single contract (loader contract that loads the blobs) to call into.
async fn deploy_chunked(
    command: &cmd::Deploy,
    compiled: &BuiltPackage,
    salt: Salt,
    account: &ForcClientAccount,
    provider: &Provider,
    pkg_name: &str,
) -> anyhow::Result<ContractId> {
    println_action_green("Deploying", &format!("contract {pkg_name} chunks"));

    let storage_slots = resolve_storage_slots(command, compiled)?;
    let chain_info = provider.chain_info().await?;
    let target = Target::from_str(&chain_info.name).unwrap_or(Target::testnet());
    let contract_url = match target.explorer_url() {
        Some(explorer_url) => format!("{explorer_url}/contract/0x"),
        None => "".to_string(),
    };

    let blobs = compiled
        .bytecode
        .bytes
        .chunks(MAX_CONTRACT_SIZE)
        .map(|chunk| Blob::new(chunk.to_vec()))
        .collect();

    let tx_policies = tx_policies_from_cmd(command);
    let contract_id =
        fuels::programs::contract::Contract::loader_from_blobs(blobs, salt, storage_slots)?
            .deploy(account, tx_policies)
            .await?
            .into();

    println_action_green(
        "Finished",
        &format!("deploying loader contract for {pkg_name} {contract_url}{contract_id}"),
    );

    Ok(contract_id)
}

/// Deploys a new proxy contract for the given package.
async fn deploy_new_proxy(
    command: &cmd::Deploy,
    pkg_name: &str,
    pkg_storage_slots: &[StorageSlot],
    impl_contract: &fuel_tx::ContractId,
    provider: &Provider,
    account: &ForcClientAccount,
) -> Result<ContractId> {
    abigen!(Contract(name = "ProxyContract", abi = "{\"programType\":\"contract\",\"specVersion\":\"1\",\"encodingVersion\":\"1\",\"concreteTypes\":[{\"type\":\"()\",\"concreteTypeId\":\"2e38e77b22c314a449e91fafed92a43826ac6aa403ae6a8acb6cf58239fbaf5d\"},{\"type\":\"enum standards::src5::AccessError\",\"concreteTypeId\":\"3f702ea3351c9c1ece2b84048006c8034a24cbc2bad2e740d0412b4172951d3d\",\"metadataTypeId\":1},{\"type\":\"enum standards::src5::State\",\"concreteTypeId\":\"192bc7098e2fe60635a9918afb563e4e5419d386da2bdbf0d716b4bc8549802c\",\"metadataTypeId\":2},{\"type\":\"enum std::option::Option<struct std::contract_id::ContractId>\",\"concreteTypeId\":\"0d79387ad3bacdc3b7aad9da3a96f4ce60d9a1b6002df254069ad95a3931d5c8\",\"metadataTypeId\":4,\"typeArguments\":[\"29c10735d33b5159f0c71ee1dbd17b36a3e69e41f00fab0d42e1bd9f428d8a54\"]},{\"type\":\"enum sway_libs::ownership::errors::InitializationError\",\"concreteTypeId\":\"1dfe7feadc1d9667a4351761230f948744068a090fe91b1bc6763a90ed5d3893\",\"metadataTypeId\":5},{\"type\":\"enum sway_libs::upgradability::errors::SetProxyOwnerError\",\"concreteTypeId\":\"3c6e90ae504df6aad8b34a93ba77dc62623e00b777eecacfa034a8ac6e890c74\",\"metadataTypeId\":6},{\"type\":\"str\",\"concreteTypeId\":\"8c25cb3686462e9a86d2883c5688a22fe738b0bbc85f458d2d2b5f3f667c6d5a\"},{\"type\":\"struct std::contract_id::ContractId\",\"concreteTypeId\":\"29c10735d33b5159f0c71ee1dbd17b36a3e69e41f00fab0d42e1bd9f428d8a54\",\"metadataTypeId\":9},{\"type\":\"struct sway_libs::upgradability::events::ProxyOwnerSet\",\"concreteTypeId\":\"96dd838b44f99d8ccae2a7948137ab6256c48ca4abc6168abc880de07fba7247\",\"metadataTypeId\":10},{\"type\":\"struct sway_libs::upgradability::events::ProxyTargetSet\",\"concreteTypeId\":\"1ddc0adda1270a016c08ffd614f29f599b4725407c8954c8b960bdf651a9a6c8\",\"metadataTypeId\":11}],\"metadataTypes\":[{\"type\":\"b256\",\"metadataTypeId\":0},{\"type\":\"enum standards::src5::AccessError\",\"metadataTypeId\":1,\"components\":[{\"name\":\"NotOwner\",\"typeId\":\"2e38e77b22c314a449e91fafed92a43826ac6aa403ae6a8acb6cf58239fbaf5d\"}]},{\"type\":\"enum standards::src5::State\",\"metadataTypeId\":2,\"components\":[{\"name\":\"Uninitialized\",\"typeId\":\"2e38e77b22c314a449e91fafed92a43826ac6aa403ae6a8acb6cf58239fbaf5d\"},{\"name\":\"Initialized\",\"typeId\":3},{\"name\":\"Revoked\",\"typeId\":\"2e38e77b22c314a449e91fafed92a43826ac6aa403ae6a8acb6cf58239fbaf5d\"}]},{\"type\":\"enum std::identity::Identity\",\"metadataTypeId\":3,\"components\":[{\"name\":\"Address\",\"typeId\":8},{\"name\":\"ContractId\",\"typeId\":9}]},{\"type\":\"enum std::option::Option\",\"metadataTypeId\":4,\"components\":[{\"name\":\"None\",\"typeId\":\"2e38e77b22c314a449e91fafed92a43826ac6aa403ae6a8acb6cf58239fbaf5d\"},{\"name\":\"Some\",\"typeId\":7}],\"typeParameters\":[7]},{\"type\":\"enum sway_libs::ownership::errors::InitializationError\",\"metadataTypeId\":5,\"components\":[{\"name\":\"CannotReinitialized\",\"typeId\":\"2e38e77b22c314a449e91fafed92a43826ac6aa403ae6a8acb6cf58239fbaf5d\"}]},{\"type\":\"enum sway_libs::upgradability::errors::SetProxyOwnerError\",\"metadataTypeId\":6,\"components\":[{\"name\":\"CannotUninitialize\",\"typeId\":\"2e38e77b22c314a449e91fafed92a43826ac6aa403ae6a8acb6cf58239fbaf5d\"}]},{\"type\":\"generic T\",\"metadataTypeId\":7},{\"type\":\"struct std::address::Address\",\"metadataTypeId\":8,\"components\":[{\"name\":\"bits\",\"typeId\":0}]},{\"type\":\"struct std::contract_id::ContractId\",\"metadataTypeId\":9,\"components\":[{\"name\":\"bits\",\"typeId\":0}]},{\"type\":\"struct sway_libs::upgradability::events::ProxyOwnerSet\",\"metadataTypeId\":10,\"components\":[{\"name\":\"new_proxy_owner\",\"typeId\":2}]},{\"type\":\"struct sway_libs::upgradability::events::ProxyTargetSet\",\"metadataTypeId\":11,\"components\":[{\"name\":\"new_target\",\"typeId\":9}]}],\"functions\":[{\"inputs\":[],\"name\":\"proxy_target\",\"output\":\"0d79387ad3bacdc3b7aad9da3a96f4ce60d9a1b6002df254069ad95a3931d5c8\",\"attributes\":[{\"name\":\"doc-comment\",\"arguments\":[\" Returns the target contract of the proxy contract.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Returns\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * [Option<ContractId>] - The new proxy contract to which all fallback calls will be passed or `None`.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Number of Storage Accesses\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * Reads: `1`\"]},{\"name\":\"storage\",\"arguments\":[\"read\"]}]},{\"inputs\":[{\"name\":\"new_target\",\"concreteTypeId\":\"29c10735d33b5159f0c71ee1dbd17b36a3e69e41f00fab0d42e1bd9f428d8a54\"}],\"name\":\"set_proxy_target\",\"output\":\"2e38e77b22c314a449e91fafed92a43826ac6aa403ae6a8acb6cf58239fbaf5d\",\"attributes\":[{\"name\":\"doc-comment\",\"arguments\":[\" Change the target contract of the proxy contract.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Additional Information\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" This method can only be called by the `proxy_owner`.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Arguments\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * `new_target`: [ContractId] - The new proxy contract to which all fallback calls will be passed.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Reverts\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * When not called by `proxy_owner`.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Number of Storage Accesses\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * Reads: `1`\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * Write: `1`\"]},{\"name\":\"storage\",\"arguments\":[\"read\",\"write\"]}]},{\"inputs\":[],\"name\":\"proxy_owner\",\"output\":\"192bc7098e2fe60635a9918afb563e4e5419d386da2bdbf0d716b4bc8549802c\",\"attributes\":[{\"name\":\"doc-comment\",\"arguments\":[\" Returns the owner of the proxy contract.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Returns\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * [State] - Represents the state of ownership for this contract.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Number of Storage Accesses\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * Reads: `1`\"]},{\"name\":\"storage\",\"arguments\":[\"read\"]}]},{\"inputs\":[],\"name\":\"initialize_proxy\",\"output\":\"2e38e77b22c314a449e91fafed92a43826ac6aa403ae6a8acb6cf58239fbaf5d\",\"attributes\":[{\"name\":\"doc-comment\",\"arguments\":[\" Initializes the proxy contract.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Additional Information\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" This method sets the storage values using the values of the configurable constants `INITIAL_TARGET` and `INITIAL_OWNER`.\"]},{\"name\":\"doc-comment\",\"arguments\":[\" This then allows methods that write to storage to be called.\"]},{\"name\":\"doc-comment\",\"arguments\":[\" This method can only be called once.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Reverts\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * When `storage::SRC14.proxy_owner` is not [State::Uninitialized].\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Number of Storage Accesses\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * Writes: `2`\"]},{\"name\":\"storage\",\"arguments\":[\"write\"]}]},{\"inputs\":[{\"name\":\"new_proxy_owner\",\"concreteTypeId\":\"192bc7098e2fe60635a9918afb563e4e5419d386da2bdbf0d716b4bc8549802c\"}],\"name\":\"set_proxy_owner\",\"output\":\"2e38e77b22c314a449e91fafed92a43826ac6aa403ae6a8acb6cf58239fbaf5d\",\"attributes\":[{\"name\":\"doc-comment\",\"arguments\":[\" Changes proxy ownership to the passed State.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Additional Information\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" This method can be used to transfer ownership between Identities or to revoke ownership.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Arguments\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * `new_proxy_owner`: [State] - The new state of the proxy ownership.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Reverts\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * When the sender is not the current proxy owner.\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * When the new state of the proxy ownership is [State::Uninitialized].\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Number of Storage Accesses\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * Reads: `1`\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * Writes: `1`\"]},{\"name\":\"storage\",\"arguments\":[\"write\"]}]}],\"loggedTypes\":[{\"logId\":\"4571204900286667806\",\"concreteTypeId\":\"3f702ea3351c9c1ece2b84048006c8034a24cbc2bad2e740d0412b4172951d3d\"},{\"logId\":\"2151606668983994881\",\"concreteTypeId\":\"1ddc0adda1270a016c08ffd614f29f599b4725407c8954c8b960bdf651a9a6c8\"},{\"logId\":\"2161305517876418151\",\"concreteTypeId\":\"1dfe7feadc1d9667a4351761230f948744068a090fe91b1bc6763a90ed5d3893\"},{\"logId\":\"4354576968059844266\",\"concreteTypeId\":\"3c6e90ae504df6aad8b34a93ba77dc62623e00b777eecacfa034a8ac6e890c74\"},{\"logId\":\"10870989709723147660\",\"concreteTypeId\":\"96dd838b44f99d8ccae2a7948137ab6256c48ca4abc6168abc880de07fba7247\"},{\"logId\":\"10098701174489624218\",\"concreteTypeId\":\"8c25cb3686462e9a86d2883c5688a22fe738b0bbc85f458d2d2b5f3f667c6d5a\"}],\"messagesTypes\":[],\"configurables\":[{\"name\":\"INITIAL_TARGET\",\"concreteTypeId\":\"0d79387ad3bacdc3b7aad9da3a96f4ce60d9a1b6002df254069ad95a3931d5c8\",\"offset\":13368},{\"name\":\"INITIAL_OWNER\",\"concreteTypeId\":\"192bc7098e2fe60635a9918afb563e4e5419d386da2bdbf0d716b4bc8549802c\",\"offset\":13320}]}",));
    let proxy_dir_output = create_proxy_contract(pkg_name)?;
    let address = account.address();

    // Add the combined storage slots from the original contract and the proxy contract.
    let proxy_storage_path = proxy_dir_output.join("proxy-storage_slots.json");
    let storage_configuration = StorageConfiguration::default()
        .add_slot_overrides(pkg_storage_slots.iter().cloned())
        .add_slot_overrides_from_file(proxy_storage_path)?;

    let configurables = ProxyContractConfigurables::default()
        .with_INITIAL_TARGET(Some(*impl_contract))?
        .with_INITIAL_OWNER(State::Initialized(Address::from(address).into()))?;

    let configuration = LoadConfiguration::default()
        .with_storage_configuration(storage_configuration)
        .with_configurables(configurables);

    let tx_policies = tx_policies_from_cmd(command);
    let proxy_contract_id: ContractId = fuels::programs::contract::Contract::load_from(
        proxy_dir_output.join("proxy.bin"),
        configuration,
    )?
    .deploy(account, tx_policies)
    .await?
    .into();

    let chain_info = provider.chain_info().await?;
    let target = Target::from_str(&chain_info.name).unwrap_or(Target::testnet());
    let contract_url = match target.explorer_url() {
        Some(explorer_url) => format!("{explorer_url}/contract/0x"),
        None => "".to_string(),
    };

    println_action_green(
        "Finished",
        &format!("deploying proxy contract for {pkg_name} {contract_url}{proxy_contract_id}"),
    );

    let proxy_contract_bech_id: Bech32ContractId = proxy_contract_id.into();
    let instance = ProxyContract::new(&proxy_contract_bech_id, account.clone());
    instance.methods().initialize_proxy().call().await?;
    println_action_green("Initialized", &format!("proxy contract for {pkg_name}"));
    Ok(proxy_contract_id)
}

/// Builds and deploys contract(s). If the given path corresponds to a workspace, all deployable members
/// will be built and deployed.
///
/// Upon success, returns the ID of each deployed contract in order of deployment.
///
/// When deploying a single contract, only that contract's ID is returned.
pub async fn deploy(command: cmd::Deploy) -> Result<Vec<DeployedContract>> {
    if command.unsigned {
        println_warning("--unsigned flag is deprecated, please prefer using --default-signer. Assuming `--default-signer` is passed. This means your transaction will be signed by an account that is funded by fuel-core by default for testing purposes.");
    }

    let mut deployed_contracts = Vec::new();
    let curr_dir = if let Some(ref path) = command.pkg.path {
        PathBuf::from(path)
    } else {
        std::env::current_dir()?
    };

    let build_opts = build_opts_from_cmd(&command);
    let built_pkgs = built_pkgs(&curr_dir, &build_opts)?;
    let pkgs_to_deploy = built_pkgs
        .iter()
        .filter(|pkg| {
            pkg.descriptor
                .manifest_file
                .check_program_type(&[TreeType::Contract])
                .is_ok()
        })
        .collect::<Vec<_>>();

    if pkgs_to_deploy.is_empty() {
        println_warning("No deployable contracts found in the current directory.");
        return Ok(deployed_contracts);
    }

    let contract_salt_map = if let Some(salt_input) = &command.salt {
        // If we're building 1 package, we just parse the salt as a string, ie. 0x00...
        // If we're building >1 package, we must parse the salt as a pair of strings, ie. contract_name:0x00...
        if built_pkgs.len() > 1 {
            let map = validate_and_parse_salts(
                salt_input,
                built_pkgs.iter().map(|b| &b.descriptor.manifest_file),
            )?;

            Some(map)
        } else {
            if salt_input.len() > 1 {
                bail!("More than 1 salt was specified when deploying a single contract");
            }

            // OK to index into salt_input and built_pkgs_with_manifest here,
            // since both are known to be len 1.

            let salt = salt_input[0]
                .parse::<Salt>()
                .map_err(|e| anyhow::anyhow!(e))
                .unwrap();
            let mut contract_salt_map = ContractSaltMap::default();
            contract_salt_map.insert(
                built_pkgs[0]
                    .descriptor
                    .manifest_file
                    .project_name()
                    .to_string(),
                salt,
            );
            Some(contract_salt_map)
        }
    } else {
        None
    };

    // Ensure that all packages are being deployed to the same node.
    let node_url = get_node_url(
        &command.node,
        &pkgs_to_deploy[0].descriptor.manifest_file.network,
    )?;
    if !pkgs_to_deploy.iter().all(|pkg| {
        get_node_url(&command.node, &pkg.descriptor.manifest_file.network).ok()
            == Some(node_url.clone())
    }) {
        bail!("All contracts in a deployment should be deployed to the same node. Please ensure that the network specified in the Forc.toml files of all contracts is the same.");
    }

    // Confirmation step. Summarize the transaction(s) for the deployment.
    let (provider, account) =
        confirm_transaction_details(&pkgs_to_deploy, &command, node_url.clone()).await?;

    for pkg in pkgs_to_deploy {
        let salt = match (&contract_salt_map, command.default_salt) {
            (Some(map), false) => {
                if let Some(salt) = map.get(pkg.descriptor.manifest_file.project_name()) {
                    *salt
                } else {
                    Default::default()
                }
            }
            (None, true) => Default::default(),
            (None, false) => rand::random(),
            (Some(_), true) => {
                bail!("Both `--salt` and `--default-salt` were specified: must choose one")
            }
        };
        let bytecode_size = pkg.bytecode.bytes.len();
        let deployed_contract_id = if bytecode_size > MAX_CONTRACT_SIZE {
            // Deploy chunked
            let node_url = get_node_url(&command.node, &pkg.descriptor.manifest_file.network)?;
            let provider = Provider::connect(node_url).await?;

            deploy_chunked(
                &command,
                pkg,
                salt,
                &account,
                &provider,
                &pkg.descriptor.name,
            )
            .await?
        } else {
            deploy_pkg(&command, pkg, salt, &provider, &account).await?
        };

        let proxy_id = match &pkg.descriptor.manifest_file.proxy {
            Some(forc_pkg::manifest::Proxy {
                enabled: true,
                address: Some(proxy_addr),
            }) => {
                // Make a call into the contract to update impl contract address to 'deployed_contract'.

                // Create a contract instance for the proxy contract using default proxy contract abi and
                // specified address.
                let proxy_contract =
                    ContractId::from_str(proxy_addr).map_err(|e| anyhow::anyhow!(e))?;

                update_proxy_contract_target(&account, proxy_contract, deployed_contract_id)
                    .await?;
                Some(proxy_contract)
            }
            Some(forc_pkg::manifest::Proxy {
                enabled: true,
                address: None,
            }) => {
                let pkg_name = &pkg.descriptor.name;
                let pkg_storage_slots = &pkg.storage_slots;
                // Deploy a new proxy contract.
                let deployed_proxy_contract = deploy_new_proxy(
                    &command,
                    pkg_name,
                    pkg_storage_slots,
                    &deployed_contract_id,
                    &provider,
                    &account,
                )
                .await?;

                // Update manifest file such that the proxy address field points to the new proxy contract.
                update_proxy_address_in_manifest(
                    &format!("0x{}", deployed_proxy_contract),
                    &pkg.descriptor.manifest_file,
                )?;
                Some(deployed_proxy_contract)
            }
            // Proxy not enabled.
            _ => None,
        };

        let deployed_contract = DeployedContract {
            id: deployed_contract_id,
            proxy: proxy_id,
            chunked: bytecode_size > MAX_CONTRACT_SIZE,
        };
        deployed_contracts.push(deployed_contract);
    }
    Ok(deployed_contracts)
}

/// Prompt the user to confirm the transactions required for deployment, as well as the signing key.
async fn confirm_transaction_details(
    pkgs_to_deploy: &[&Arc<BuiltPackage>],
    command: &cmd::Deploy,
    node_url: String,
) -> Result<(Provider, ForcClientAccount)> {
    // Confirmation step. Summarize the transaction(s) for the deployment.
    let mut tx_count = 0;
    let tx_summary = pkgs_to_deploy
        .iter()
        .map(|pkg| {
            tx_count += 1;
            let proxy_text = match &pkg.descriptor.manifest_file.proxy {
                Some(forc_pkg::manifest::Proxy {
                    enabled: true,
                    address,
                }) => {
                    tx_count += 1;
                    if address.is_some() {
                        " + update proxy"
                    } else {
                        " + deploy proxy"
                    }
                }
                _ => "",
            };

            let pkg_bytecode_len = pkg.bytecode.bytes.len();
            let blob_text = if pkg_bytecode_len > MAX_CONTRACT_SIZE {
                let number_of_blobs = pkg_bytecode_len.div_ceil(MAX_CONTRACT_SIZE);
                tx_count += number_of_blobs;
                &format!(" + {number_of_blobs} blobs")
            } else {
                ""
            };

            format!(
                "deploy {}{blob_text}{proxy_text}",
                pkg.descriptor.manifest_file.project_name()
            )
        })
        .collect::<Vec<_>>()
        .join(" + ");

    println_action_green("Confirming", &format!("transactions [{tx_summary}]"));
    println_action_green("", &format!("Network: {node_url}"));

    let provider = Provider::connect(node_url.clone()).await?;

    let wallet_mode = if command.default_signer || command.signing_key.is_some() {
        SignerSelectionMode::Manual
    } else if let Some(arn) = &command.aws_kms_signer {
        SignerSelectionMode::AwsSigner(arn.clone())
    } else {
        println_action_green("", &format!("Wallet: {}", default_wallet_path().display()));
        let password = prompt_forc_wallet_password()?;
        SignerSelectionMode::ForcWallet(password)
    };

    // TODO: Display the estimated gas cost of the transaction(s).
    // https://github.com/FuelLabs/sway/issues/6277

    let account = select_account(
        &wallet_mode,
        command.default_signer || command.unsigned,
        command.signing_key,
        &provider,
        tx_count,
    )
    .await?;

    Ok((provider.clone(), account))
}

/// Deploy a single pkg given deploy command and the manifest file
pub async fn deploy_pkg(
    command: &cmd::Deploy,
    compiled: &BuiltPackage,
    salt: Salt,
    provider: &Provider,
    account: &ForcClientAccount,
) -> Result<fuel_tx::ContractId> {
    let manifest = &compiled.descriptor.manifest_file;
    let node_url = provider.url();
    let client = FuelClient::new(node_url)?;

    let bytecode = &compiled.bytecode.bytes;

    let storage_slots = resolve_storage_slots(command, compiled)?;
    let contract = Contract::from(bytecode.clone());
    let root = contract.root();
    let state_root = Contract::initial_state_root(storage_slots.iter());
    let contract_id = contract.id(&salt, &root, &state_root);
    let tx_policies = tx_policies_from_cmd(command);

    let mut tb = CreateTransactionBuilder::prepare_contract_deployment(
        bytecode.clone(),
        contract_id,
        state_root,
        salt,
        storage_slots.clone(),
        tx_policies,
    );

    account.add_witnesses(&mut tb)?;
    account.adjust_for_fee(&mut tb, 0).await?;

    let tx = tb.build(provider).await?;
    let tx = Transaction::from(tx);

    let chain_info = client.chain_info().await?;
    let chain_id = chain_info.consensus_parameters.chain_id();

    // If only submitting the transaction, don't wait for the deployment to complete
    let contract_id: ContractId = if command.submit_only {
        match client.submit(&tx).await {
            Ok(transaction_id) => {
                // Create a deployment artifact.
                create_deployment_artifact(
                    DeploymentArtifact {
                        transaction_id: format!("0x{}", transaction_id),
                        salt: format!("0x{}", salt),
                        network_endpoint: node_url.to_string(),
                        chain_id,
                        contract_id: format!("0x{}", contract_id),
                        deployment_size: bytecode.len(),
                        deployed_block_height: None,
                    },
                    command,
                    manifest,
                    chain_info,
                )?;

                contract_id
            }
            Err(e) => {
                bail!(
                    "contract {} failed to deploy due to an error: {:?}",
                    &contract_id,
                    e
                )
            }
        }
    } else {
        let deployment_request = client.submit_and_await_commit(&tx).map(|res| match res {
            Ok(logs) => match logs {
                TransactionStatus::Submitted { .. } => {
                    bail!("contract {} deployment timed out", &contract_id);
                }
                TransactionStatus::Success { block_height, .. } => {
                    // Create a deployment artifact.
                    create_deployment_artifact(
                        DeploymentArtifact {
                            transaction_id: format!("0x{}", tx.id(&chain_id)),
                            salt: format!("0x{}", salt),
                            network_endpoint: node_url.to_string(),
                            chain_id,
                            contract_id: format!("0x{}", contract_id),
                            deployment_size: bytecode.len(),
                            deployed_block_height: Some(*block_height),
                        },
                        command,
                        manifest,
                        chain_info,
                    )?;

                    Ok(contract_id)
                }
                e => {
                    bail!(
                        "contract {} failed to deploy due to an error: {:?}",
                        &contract_id,
                        e
                    )
                }
            },
            Err(e) => bail!("{e}"),
        });
        tokio::time::timeout(
            Duration::from_millis(TX_SUBMIT_TIMEOUT_MS),
            deployment_request,
        )
            .await
            .with_context(|| {
                format!(
                    "Timed out waiting for contract {} to deploy. The transaction may have been dropped.",
                    &contract_id
                )
            })??
    };

    Ok(contract_id)
}

fn tx_policies_from_cmd(cmd: &cmd::Deploy) -> TxPolicies {
    let mut tx_policies = TxPolicies::default();
    if let Some(max_fee) = cmd.gas.max_fee {
        tx_policies = tx_policies.with_max_fee(max_fee);
    }
    if let Some(script_gas_limit) = cmd.gas.script_gas_limit {
        tx_policies = tx_policies.with_script_gas_limit(script_gas_limit);
    }
    tx_policies
}

fn build_opts_from_cmd(cmd: &cmd::Deploy) -> pkg::BuildOpts {
    pkg::BuildOpts {
        pkg: pkg::PkgOpts {
            path: cmd.pkg.path.clone(),
            offline: cmd.pkg.offline,
            terse: cmd.pkg.terse,
            locked: cmd.pkg.locked,
            output_directory: cmd.pkg.output_directory.clone(),
            ipfs_node: cmd.pkg.ipfs_node.clone().unwrap_or_default(),
        },
        print: pkg::PrintOpts {
            ast: cmd.print.ast,
            dca_graph: cmd.print.dca_graph.clone(),
            dca_graph_url_format: cmd.print.dca_graph_url_format.clone(),
            asm: cmd.print.asm(),
            bytecode: cmd.print.bytecode,
            bytecode_spans: false,
            ir: cmd.print.ir(),
            reverse_order: cmd.print.reverse_order,
        },
        time_phases: cmd.print.time_phases,
        metrics_outfile: cmd.print.metrics_outfile.clone(),
        minify: pkg::MinifyOpts {
            json_abi: cmd.minify.json_abi,
            json_storage_slots: cmd.minify.json_storage_slots,
        },
        build_profile: cmd.build_profile.clone(),
        release: cmd.build_profile == BuildProfile::RELEASE,
        error_on_warnings: false,
        binary_outfile: cmd.build_output.bin_file.clone(),
        debug_outfile: cmd.build_output.debug_file.clone(),
        build_target: BuildTarget::default(),
        tests: false,
        member_filter: pkg::MemberFilter::only_contracts(),
        experimental: ExperimentalFlags {
            new_encoding: !cmd.no_encoding_v1,
        },
    }
}

/// Creates a deployment artifact and writes it to a file.
///
/// This function is used to generate a deployment artifact containing details
/// about the deployment, such as the transaction ID, salt, network endpoint,
/// chain ID, contract ID, deployment size, and deployed block height. It then
/// writes this artifact to a specified output directory.
fn create_deployment_artifact(
    deployment_artifact: DeploymentArtifact,
    cmd: &cmd::Deploy,
    manifest: &PackageManifestFile,
    chain_info: ChainInfo,
) -> Result<()> {
    let contract_id = ContractId::from_str(&deployment_artifact.contract_id).unwrap();
    let pkg_name = manifest.project_name();

    let target = Target::from_str(&chain_info.name).unwrap_or(Target::testnet());
    let (contract_url, block_url) = match target.explorer_url() {
        Some(explorer_url) => (
            format!("{explorer_url}/contract/0x"),
            format!("{explorer_url}/block/"),
        ),
        None => ("".to_string(), "".to_string()),
    };
    println_action_green(
        "Finished",
        &format!("deploying {pkg_name} {contract_url}{contract_id}"),
    );

    let block_height = deployment_artifact.deployed_block_height;
    if let Some(block_height) = block_height {
        println_action_green("Deployed", &format!("in block {block_url}{block_height}"));
    }

    let output_dir = cmd
        .pkg
        .output_directory
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| default_output_directory(manifest.dir()))
        .join("deployments");
    deployment_artifact.to_file(&output_dir, pkg_name, contract_id)
}

#[cfg(test)]
mod test {
    use super::*;

    fn setup_manifest_files() -> BTreeMap<String, PackageManifestFile> {
        let mut contract_to_manifest = BTreeMap::default();

        let manifests_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("test")
            .join("data");

        for entry in manifests_dir.read_dir().unwrap() {
            let manifest =
                PackageManifestFile::from_file(entry.unwrap().path().join("Forc.toml")).unwrap();
            contract_to_manifest.insert(manifest.project_name().to_string(), manifest);
        }

        contract_to_manifest
    }

    #[test]
    fn test_parse_and_validate_salts_pass() {
        let mut manifests = setup_manifest_files();
        let mut expected = ContractSaltMap::new();
        let mut salt_strs = vec![];

        // Remove contracts with dependencies
        manifests.remove("contract_with_dep_with_salt_conflict");
        manifests.remove("contract_with_dep");

        for (index, manifest) in manifests.values().enumerate() {
            let salt = "0x0000000000000000000000000000000000000000000000000000000000000000";

            let salt_str = format!("{}:{salt}", manifest.project_name());
            salt_strs.push(salt_str.to_string());

            expected.insert(
                manifest.project_name().to_string(),
                salt.parse::<Salt>().unwrap(),
            );

            let got = validate_and_parse_salts(&salt_strs, manifests.values()).unwrap();
            assert_eq!(got.len(), index + 1);
            assert_eq!(got, expected);
        }
    }

    #[test]
    fn test_parse_and_validate_salts_duplicate_salt_input() {
        let manifests = setup_manifest_files();
        let first_name = manifests.first_key_value().unwrap().0;
        let salt: Salt = "0x0000000000000000000000000000000000000000000000000000000000000000"
            .parse()
            .unwrap();
        let salt_str = format!("{first_name}:{salt}");
        let err_message =
            format!("2 salts provided for contract '{first_name}':\n  {salt}\n  {salt}");

        assert_eq!(
            validate_and_parse_salts(&[salt_str.clone(), salt_str], manifests.values())
                .unwrap_err()
                .to_string(),
            err_message,
        );
    }

    #[test]
    fn test_parse_single_salt_multiple_manifests_malformed_input() {
        let manifests = setup_manifest_files();
        let salt_str =
            "contract_a=0x0000000000000000000000000000000000000000000000000000000000000000";
        let err_message =
            "Invalid salt provided - salt must be in the form <CONTRACT_NAME>:<SALT> when deploying a workspace";

        assert_eq!(
            validate_and_parse_salts(&[salt_str.to_string()], manifests.values())
                .unwrap_err()
                .to_string(),
            err_message,
        );
    }

    #[test]
    fn test_parse_multiple_salts_conflict() {
        let manifests = setup_manifest_files();
        let salt_str =
            "contract_with_dep:0x0000000000000000000000000000000000000000000000000000000000000001";
        let err_message =
            "Redeclaration of salt using the option '--salt' while a salt exists for contract 'contract_with_dep' \
            under the contract dependencies of the Forc.toml manifest for 'contract_with_dep_with_salt_conflict'\n\
            Existing salt: '0x0000000000000000000000000000000000000000000000000000000000000000',\n\
            You declared: '0x0000000000000000000000000000000000000000000000000000000000000001'\n";

        assert_eq!(
            validate_and_parse_salts(&[salt_str.to_string()], manifests.values())
                .unwrap_err()
                .to_string(),
            err_message,
        );
    }
}
