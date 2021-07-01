use core_lang::{compile_to_bytecode, parse, BuildConfig, Namespace};
use fuel_tx::{crypto::hash, ContractAddress, Output, Salt, Transaction};

use crate::cli::DeployCommand;

use crate::utils::{constants, helpers};
use constants::MANIFEST_FILE_NAME;
use helpers::{find_manifest_dir, get_main_file, read_manifest};
use std::{fmt, io, path::PathBuf};

use super::forc_build::compile_dependency_lib;

pub fn deploy(_: DeployCommand) -> Result<(), DeployError> {
    let curr_dir = std::env::current_dir()?;

    match find_manifest_dir(&curr_dir) {
        Some(manifest_dir) => {
            let build_config = BuildConfig::root_from_manifest_path(manifest_dir.clone());
            let manifest = read_manifest(&manifest_dir)?;
            let mut namespace: Namespace = Default::default();

            // compile dependencies
            if let Some(ref deps) = manifest.dependencies {
                for (dependency_name, dependency_details) in deps.iter() {
                    compile_dependency_lib(
                        &curr_dir,
                        &dependency_name,
                        &dependency_details,
                        &mut namespace,
                    )?;
                }
            }

            // compile this program with all of its dependencies
            let main_file = get_main_file(&manifest, &manifest_dir)?;

            // parse it and check is it a contract
            match parse(main_file) {
                core_lang::CompileResult::Ok {
                    value: parse_tree,
                    warnings: _,
                    errors: _,
                } => {
                    if let Some(_) = &parse_tree.contract_ast {
                        // create Transaction::Create from contract file
                        let compiled_contract =
                            compile_contract(main_file, namespace, build_config)?;
                        let tx = create_contract_tx(compiled_contract);
                        // todo: pass the transaction to the running node
                        println!("{:?}", tx);

                        Ok(())
                    } else {
                        Err("Project is not a contract".into())
                    }
                }
                _ => Err("Project does not compile".into()),
            }
        }
        None => Err(DeployError::manifest_file_missing(curr_dir)),
    }
}

fn compile_contract(
    contract_file: &str,
    namespace: Namespace,
    build_config: BuildConfig,
) -> Result<Vec<u8>, DeployError> {
    let result = compile_to_bytecode(contract_file, &namespace, build_config);

    match result {
        core_lang::BytecodeCompilationResult::Success { bytes, warnings: _ } => Ok(bytes),
        _ => Err("Failed to compile".into()),
    }
}

fn create_contract_tx(compiled_contract: Vec<u8>) -> Transaction {
    let gas_price = 0;
    let gas_limit = 10000000;
    let maturity = 0;
    let bytecode_witness_index = 0;
    let witnesses = vec![compiled_contract.into()];

    let salt = Salt::new([0; 32]);
    let static_contracts = vec![];
    let inputs = vec![];

    let zero_hash = hash("0".as_bytes());

    let outputs = vec![Output::ContractCreated {
        contract_id: ContractAddress::new(zero_hash.into()),
    }];

    Transaction::create(
        gas_price,
        gas_limit,
        maturity,
        bytecode_witness_index,
        salt,
        static_contracts,
        inputs,
        outputs,
        witnesses,
    )
}

pub struct DeployError {
    pub message: String,
}

impl DeployError {
    fn manifest_file_missing(curr_dir: PathBuf) -> Self {
        let message = format!(
            "Manifest file not found at {:?}. Project root should contain '{}'",
            curr_dir, MANIFEST_FILE_NAME
        );
        Self { message }
    }
}

impl fmt::Display for DeployError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}

impl From<&str> for DeployError {
    fn from(s: &str) -> Self {
        DeployError {
            message: s.to_string(),
        }
    }
}

impl From<String> for DeployError {
    fn from(s: String) -> Self {
        DeployError { message: s }
    }
}

impl From<io::Error> for DeployError {
    fn from(e: io::Error) -> Self {
        DeployError {
            message: e.to_string(),
        }
    }
}
