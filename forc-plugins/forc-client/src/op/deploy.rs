use crate::{
    cmd,
    constants::TX_SUBMIT_TIMEOUT_MS,
    util::{
        node_url::get_node_url,
        pkg::built_pkgs,
        target::Target,
        tx::{prompt_forc_wallet_password, select_secret_key, WalletSelectionMode},
    },
};
use anyhow::{bail, Context, Result};
use forc_pkg::manifest::GenericManifestFile;
use forc_pkg::{self as pkg, PackageManifestFile};
use forc_tracing::{println_action_green, println_warning};
use forc_util::default_output_directory;
use forc_wallet::utils::default_wallet_path;
use fuel_core_client::client::types::TransactionStatus;
use fuel_core_client::client::FuelClient;
use fuel_crypto::fuel_types::ChainId;
use fuel_tx::{Salt, Transaction};
use fuel_vm::prelude::*;
use fuels_accounts::{provider::Provider, wallet::WalletUnlocked, Account};
use fuels_core::types::{transaction::TxPolicies, transaction_builders::CreateTransactionBuilder};
use futures::FutureExt;
use pkg::{manifest::build_profile::ExperimentalFlags, BuildProfile, BuiltPackage};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    str::FromStr,
};
use std::{sync::Arc, time::Duration};
use sway_core::language::parsed::TreeType;
use sway_core::BuildTarget;

#[derive(Debug, PartialEq, Eq)]
pub struct DeployedContract {
    pub id: fuel_tx::ContractId,
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

    let mut contract_ids = Vec::new();
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
        return Ok(contract_ids);
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
    let (provider, signing_key) =
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
        let contract_id = deploy_pkg(
            &command,
            pkg,
            salt,
            &provider,
            &signing_key,
            node_url.clone(),
        )
        .await?;
        contract_ids.push(contract_id);
    }
    Ok(contract_ids)
}

/// Prompt the user to confirm the transactions required for deployment, as well as the signing key.
async fn confirm_transaction_details(
    pkgs_to_deploy: &[&Arc<BuiltPackage>],
    command: &cmd::Deploy,
    node_url: String,
) -> Result<(Provider, SecretKey)> {
    // Confirmation step. Summarize the transaction(s) for the deployment.
    let tx_summary = pkgs_to_deploy
        .iter()
        .map(|pkg| format!("deploy {}", pkg.descriptor.manifest_file.project_name()))
        .collect::<Vec<_>>()
        .join(" + ");

    let tx_count = pkgs_to_deploy.len();

    println_action_green("Confirming", &format!("transactions [{tx_summary}]"));
    println_action_green("", &format!("Network: {node_url}"));

    let provider = Provider::connect(node_url.clone()).await?;

    let wallet_mode = if command.default_signer || command.signing_key.is_some() {
        WalletSelectionMode::Manual
    } else {
        println_action_green("", &format!("Wallet: {}", default_wallet_path().display()));
        let password = prompt_forc_wallet_password()?;
        WalletSelectionMode::ForcWallet(password)
    };

    // TODO: Display the estimated gas cost of the transaction(s).
    // https://github.com/FuelLabs/sway/issues/6277

    let signing_key = select_secret_key(
        &wallet_mode,
        command.default_signer || command.unsigned,
        command.signing_key,
        &provider,
        tx_count,
    )
    .await?
    .ok_or_else(|| anyhow::anyhow!("failed to select a signer for the transaction"))?;

    Ok((provider.clone(), signing_key))
}

/// Deploy a single pkg given deploy command and the manifest file
pub async fn deploy_pkg(
    command: &cmd::Deploy,
    compiled: &BuiltPackage,
    salt: Salt,
    provider: &Provider,
    signing_key: &SecretKey,
    node_url: String,
) -> Result<DeployedContract> {
    let manifest = &compiled.descriptor.manifest_file;
    let client = FuelClient::new(node_url.clone())?;

    let bytecode = &compiled.bytecode.bytes;

    let mut storage_slots =
        if let Some(storage_slot_override_file) = &command.override_storage_slots {
            let storage_slots_file = std::fs::read_to_string(storage_slot_override_file)?;
            let storage_slots: Vec<StorageSlot> = serde_json::from_str(&storage_slots_file)?;
            storage_slots
        } else {
            compiled.storage_slots.clone()
        };
    storage_slots.sort();

    let contract = Contract::from(bytecode.clone());
    let root = contract.root();
    let state_root = Contract::initial_state_root(storage_slots.iter());
    let contract_id = contract.id(&salt, &root, &state_root);
    let tx_policies = TxPolicies::default();

    let mut tb = CreateTransactionBuilder::prepare_contract_deployment(
        bytecode.clone(),
        contract_id,
        state_root,
        salt,
        storage_slots.clone(),
        tx_policies,
    );
    let wallet = WalletUnlocked::new_from_private_key(*signing_key, Some(provider.clone()));

    wallet.add_witnesses(&mut tb)?;
    wallet.adjust_for_fee(&mut tb, 0).await?;
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

    Ok(DeployedContract { id: contract_id })
}

fn build_opts_from_cmd(cmd: &cmd::Deploy) -> pkg::BuildOpts {
    pkg::BuildOpts {
        pkg: pkg::PkgOpts {
            path: cmd.pkg.path.clone(),
            offline: cmd.pkg.offline,
            terse: cmd.pkg.terse,
            locked: cmd.pkg.locked,
            output_directory: cmd.pkg.output_directory.clone(),
            json_abi_with_callpaths: cmd.pkg.json_abi_with_callpaths,
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

fn create_deployment_artifact(
    deployment_artifact: DeploymentArtifact,
    cmd: &cmd::Deploy,
    manifest: &PackageManifestFile,
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
    if block_height.is_some() {
        let block_height_formatted =
            match u32::from_str_radix(&block_height.unwrap().to_string(), 16) {
                Ok(decimal) => format!("{block_url}{decimal}"),
                Err(_) => block_height.to_string(),
            };

        println_action_green("Deployed", &format!("in block {block_height_formatted}"));
        println_action_green("Deployed", &format!("in block {block_height_formatted}"));
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
