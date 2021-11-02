use crate::abi_encoder::ABIEncoder;
use crate::errors::Error;
use crate::tokens::{Detokenize, Token};
use crate::types::{Function, Selector};
use forc::test::{forc_build, BuildCommand};
use forc::util::helpers::{find_manifest_dir, read_manifest};
use forc::util::{constants, start_fuel_core};
use fuel_client::client::FuelClient;
use fuel_tx::{Input, Output, Receipt, Salt, Transaction};
use fuel_vm::prelude::Contract as FuelContract;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::process::Child;

use std::marker::PhantomData;

pub type ContractID = String;

#[derive(Debug)]
pub struct CompiledContract {
    raw: Vec<u8>,
    inputs: Vec<Input>,
    outputs: Vec<Output>,
    target_network_url: String,
}

/// Contract is a struct to interface with a contract. That includes things such as
/// compiling, deploying, and running transactions against a contract.
pub struct Contract {}

impl Contract {
    pub fn new() -> Self {
        Self {}
    }

    /// Creates an ABI call based on a function selector and
    /// the encoding of its call arguments, which is a slice of Tokens.
    /// It returns a prepared ContractCall that can further be used to
    /// make the actual transaction.
    /// This method is the underlying implementation of the functions
    /// generated from an ABI JSON spec, i.e, this is what's generated:
    /// quote! {
    ///     #doc
    ///     pub fn #name(&self #input) -> #result {
    ///         Contract::method_hash(#tokenized_signature, #arg)
    ///     }
    /// }
    /// For more details see `code_gen/functions_gen.rs`.
    pub fn method_hash<D: Detokenize>(
        signature: Selector,
        args: &[Token],
    ) -> Result<ContractCall<D>, Error> {
        let mut encoder = ABIEncoder::new();

        let encoded_params = hex::encode(encoder.encode(args).unwrap());
        let encoded_selector = hex::encode(signature);

        // Temporarily printing the encoded selector+params to stdout for
        // debugging purposes.
        println!("encoded: {}{}\n", encoded_selector, encoded_params);

        // TODO: this is where we'll craft the transaction, using the actual fuel-tx
        // The actual call will likely happen in `ContractCall`.
        let tx = TransactionRequest { data: None };
        Ok(ContractCall {
            encoded_params,
            encoded_selector,
            tx,
            function: None,
            datatype: PhantomData,
        })
    }

    /// Launches a local `fuel-core` network and deploys a contract to it.
    /// If you want to deploy a contract against another network of
    /// your choosing, use the `deploy` function instead.
    /// Be careful when passing `false` to `stop_node` as it might leak.
    /// In case you want to test many deployments and interactions against the same
    /// network session, pass `true` to `stop_node` and make sure to
    /// stop running the node once you're done by unwrapping the `Option<Child>` returned
    /// and calling its `.kill()` method.
    pub async fn launch_and_deploy(
        compiled_contract: CompiledContract,
        stop_node: bool,
    ) -> Result<(Option<Child>, ContractID, Vec<Receipt>), Error> {
        let client = FuelClient::new(compiled_contract.target_network_url.clone())?;

        match client.health().await {
            // Network already up-and-running
            Ok(_) => {
                let (contract_id, receipts) = Self::deploy(compiled_contract, client).await?;
                Ok((None, contract_id, receipts))
            }
            Err(_) => {
                // Launch network
                let mut node = start_fuel_core(&compiled_contract.target_network_url, &client)
                    .await
                    .map_err(|e| {
                        Error::InfrastructureError(format!(
                            "{}. Make sure you have `fuel-core` locally installed",
                            e
                        ))
                    })?;
                let (contract_id, receipts) = Self::deploy(compiled_contract, client).await?;

                if stop_node {
                    node.kill().await.expect("Node should be killed");
                }

                Ok((Some(node), contract_id, receipts))
            }
        }
    }

    /// Deploys a compiled contract to a running node
    pub async fn deploy(
        compiled_contract: CompiledContract,
        fuel_client: FuelClient,
    ) -> Result<(ContractID, Vec<Receipt>), Error> {
        let (tx, contract_id) = Self::contract_deployment_transaction(compiled_contract);

        match fuel_client.transact(&tx).await {
            Ok(logs) => Ok((contract_id, logs)),
            Err(e) => Err(Error::TransactionError(e.to_string())),
        }
    }

    /// Compiles a Sway contract
    pub fn compile_sway_contract(project_path: &str) -> Result<CompiledContract, Error> {
        let build_command = BuildCommand {
            path: Some(project_path.into()),
            print_finalized_asm: false,
            print_intermediate_asm: false,
            binary_outfile: None,
            offline_mode: false,
            silent_mode: true,
        };

        let raw =
            forc_build::build(build_command).map_err(|message| Error::CompilationError(message))?;

        let manifest_dir = find_manifest_dir(&PathBuf::from(project_path)).unwrap();
        let manifest = read_manifest(&manifest_dir).map_err(|e| {
            Error::CompilationError(format!("Failed to find manifest for contract: {}", e))
        })?;

        let (inputs, outputs) = manifest.get_tx_inputs_and_outputs().map_err(|e| {
            Error::CompilationError(format!(
                "Failed to find contract's inputs and outputs: {}",
                e
            ))
        })?;

        let node_url = match &manifest.network {
            Some(network) => &network.url,
            _ => constants::DEFAULT_NODE_URL,
        };

        Ok(CompiledContract {
            raw,
            inputs,
            outputs,
            target_network_url: node_url.to_string(),
        })
    }

    /// Crafts a transaction used to deploy a contract
    pub fn contract_deployment_transaction(
        compiled_contract: CompiledContract,
    ) -> (Transaction, ContractID) {
        let gas_price = 0;
        let gas_limit = 10000000;
        let maturity = 0;
        let bytecode_witness_index = 0;
        let witnesses = vec![compiled_contract.raw.clone().into()];

        let salt = Salt::new([0; 32]);
        let static_contracts = vec![];

        let contract = FuelContract::from(compiled_contract.raw);
        let root = contract.root();
        let id = contract.id(&salt, &root);
        let contract_id_str = hex::encode(id);
        let outputs = [
            &[Output::ContractCreated { contract_id: id }],
            &compiled_contract.outputs[..],
        ]
        .concat();

        let tx = Transaction::create(
            gas_price,
            gas_limit,
            maturity,
            bytecode_witness_index,
            salt,
            static_contracts,
            compiled_contract.inputs,
            outputs,
            witnesses,
        );

        (tx, contract_id_str)
    }
}

/// Parameters for sending a transaction
#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct TransactionRequest {
    /// The compiled code of a contract OR the first 4 bytes of the hash of the
    /// invoked method signature and encoded parameters. For details see Ethereum Contract ABI
    pub data: Option<Vec<u8>>,
    // More later
}

#[derive(Debug, Clone)]
#[must_use = "contract calls do nothing unless you `send` or `call` them"]
/// Helper for managing a transaction before submitting it to a node
pub struct ContractCall<D> {
    /// The raw transaction object
    pub tx: TransactionRequest, // Maybe not necessary?
    /// The ABI of the function being called
    pub function: Option<Function>, // Temporarily an option
    // To be used in the future:
    // pub block: Option<BlockId>,
    // pub(crate) client: Arc<M>,
    pub datatype: PhantomData<D>,

    pub encoded_params: String,
    pub encoded_selector: String,
}

impl<D> ContractCall<D>
where
    D: Detokenize,
{
    pub fn call(&self) -> Result<D, Error> {
        unimplemented!()
    }
}
