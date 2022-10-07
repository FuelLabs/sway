use anyhow::{bail, Result};
use forc_pkg::{BuildOptions, Compiled, ManifestFile};
use fuel_crypto::Signature;
use fuel_gql_client::client::FuelClient;
use fuel_tx::{Output, Salt, StorageSlot, Transaction};
use fuel_vm::prelude::*;
use fuels_core::constants::BASE_ASSET_ID;
use fuels_signers::{provider::Provider, wallet::Wallet};
use fuels_types::bech32::Bech32Address;
use std::{io::Write, path::PathBuf, str::FromStr};
use sway_core::language::parsed::TreeType;
use sway_utils::constants::DEFAULT_NODE_URL;
use tracing::info;

use crate::ops::{deploy::cmd::DeployCommand, parameters::TxParameters};

pub async fn deploy(command: DeployCommand) -> Result<fuel_tx::ContractId> {
    let curr_dir = if let Some(ref path) = command.path {
        PathBuf::from(path)
    } else {
        std::env::current_dir()?
    };
    let manifest = ManifestFile::from_dir(&curr_dir)?;
    manifest.check_program_type(vec![TreeType::Contract])?;

    let DeployCommand {
        path,
        print_ast,
        print_finalized_asm,
        print_intermediate_asm,
        print_ir,
        binary_outfile,
        debug_outfile,
        offline_mode,
        terse_mode,
        output_directory,
        minify_json_abi,
        minify_json_storage_slots,
        locked,
        url,
        build_profile,
        release,
        time_phases,
        generate_logged_types,
        unsigned,
        gas_limit,
        gas_price,
    } = command;

    let build_options = BuildOptions {
        path,
        print_ast,
        print_finalized_asm,
        print_intermediate_asm,
        print_ir,
        binary_outfile,
        offline_mode,
        debug_outfile,
        terse_mode,
        output_directory,
        minify_json_abi,
        minify_json_storage_slots,
        locked,
        build_profile,
        release,
        time_phases,
        generate_logged_types,
    };

    let compiled = forc_pkg::build_with_options(build_options)?;

    let node_url = match &manifest.network {
        Some(network) => &network.url,
        _ => DEFAULT_NODE_URL,
    };

    let node_url = url.unwrap_or_else(|| node_url.to_string());

    let client = FuelClient::new(node_url)?;

    let (mut tx, contract_id) = if unsigned {
        create_contract_tx(
            compiled.bytecode,
            Vec::<fuel_tx::Input>::new(),
            Vec::<fuel_tx::Output>::new(),
            compiled.storage_slots,
        )
    } else {
        let mut wallet_address = String::new();
        print!(
            "Please provide the address of the wallet you are going to sign this transaction with:"
        );
        std::io::stdout().flush()?;
        std::io::stdin().read_line(&mut wallet_address)?;
        let address = Bech32Address::from_str(wallet_address.trim())?;
        let locked_wallet = Wallet::from_address(address, Some(Provider::new(client.clone())));
        let tx_parameters = TxParameters::new(gas_limit, gas_price);
        create_signed_contract_tx(compiled, locked_wallet, tx_parameters).await?
    };

    if !unsigned {
        // Ask for the signature and add it as a witness
        let mut signature = String::new();
        print!("Please provide the signature for this transaction:");
        std::io::stdout().flush()?;
        std::io::stdin().read_line(&mut signature)?;

        let signature = Signature::from_str(signature.trim())?;
        let witness = vec![Witness::from(signature.as_ref())];

        let mut witnesses: Vec<Witness> = tx.witnesses().to_vec();

        match witnesses.len() {
            0 => tx.set_witnesses(witness),
            _ => {
                witnesses.extend(witness);
                tx.set_witnesses(witnesses)
            }
        }
    }

    match client.submit(&tx).await {
        Ok(logs) => {
            info!("Logs:\n{:?}", logs);
            Ok(contract_id)
        }
        Err(e) => bail!("{e}"),
    }
}

async fn create_signed_contract_tx(
    compiled_contract: Compiled,
    signer_wallet: Wallet,
    tx_parameters: TxParameters,
) -> Result<(Transaction, fuel_tx::ContractId)> {
    let maturity = 0;
    let bytecode_witness_index = 0;
    let witnesses = vec![compiled_contract.bytecode.clone().into()];

    let salt = Salt::new([0; 32]);

    let contract = Contract::from(compiled_contract.bytecode);
    let root = contract.root();

    let mut storage_slots = compiled_contract.storage_slots;
    storage_slots.sort();
    let state_root = Contract::initial_state_root(storage_slots.iter());
    let contract_id = contract.id(&salt, &root, &state_root);
    info!("Contract id: 0x{}", hex::encode(contract_id));

    let outputs: Vec<Output> = vec![
        Output::contract_created(contract_id, state_root),
        // Note that the change will be computed by the node.
        // Here we only have to tell the node who will own the change and its asset ID.
        // For now we use the BASE_ASSET_ID constant
        Output::change(signer_wallet.address().into(), 0, BASE_ASSET_ID),
    ];
    let coin_witness_index = 1;

    let inputs = signer_wallet
        .get_asset_inputs_for_amount(AssetId::default(), 1_000_000, coin_witness_index)
        .await?;
    let tx = Transaction::create(
        tx_parameters.gas_price,
        tx_parameters.gas_limit,
        maturity,
        bytecode_witness_index,
        salt,
        storage_slots,
        inputs,
        outputs,
        witnesses,
    );

    info!("Tx id to sign {}", tx.id());
    Ok((tx, contract_id))
}

fn create_contract_tx(
    compiled_contract: Vec<u8>,
    inputs: Vec<Input>,
    outputs: Vec<Output>,
    storage_slots: Vec<StorageSlot>,
) -> (Transaction, fuel_tx::ContractId) {
    let gas_price = 0;
    let gas_limit = fuel_tx::ConsensusParameters::default().max_gas_per_tx;
    let maturity = 0;
    let bytecode_witness_index = 0;
    let witnesses = vec![compiled_contract.clone().into()];

    let salt = Salt::new([0; 32]);

    let contract = Contract::from(compiled_contract);
    let root = contract.root();

    // The VM currently requires that storage slots are sorted but this shouldn't be neessary.
    // Downstream tooling should do the sorting themselves.
    // Ref: https://github.com/FuelLabs/fuel-tx/issues/153
    let mut storage_slots = storage_slots;
    storage_slots.sort();
    let state_root = Contract::initial_state_root(storage_slots.iter());
    let id = contract.id(&salt, &root, &state_root);
    info!("Contract id: 0x{}", hex::encode(id));
    let outputs = [
        &[Output::ContractCreated {
            contract_id: id,
            state_root,
        }],
        &outputs[..],
    ]
    .concat();

    (
        Transaction::create(
            gas_price,
            gas_limit,
            maturity,
            bytecode_witness_index,
            salt,
            storage_slots,
            inputs,
            outputs,
            witnesses,
        ),
        id,
    )
}
