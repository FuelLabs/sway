use forc_pkg as pkg;
use fuel_abi_types::error_codes::ErrorSignal;
use fuel_tx as tx;
use fuel_vm::checked_transaction::builder::TransactionBuilderExt;
use fuel_vm::{self as vm, fuel_asm, prelude::Instruction};
use pkg::{Built, BuiltPackage};
use pkg::{PkgTestEntry, TestPassCondition};
use rand::{Rng, SeedableRng};
use rayon::prelude::*;
use std::ops::Deref;
use std::{collections::HashMap, fs, path::PathBuf, sync::Arc};
use sway_core::BuildTarget;
use sway_types::Span;
use tx::output::contract::Contract;
use tx::{Chargeable, Finalizable, TransactionBuilder};
use vm::interpreter::ExecutableTransaction;
use vm::prelude::SecretKey;
use vm::storage::MemoryStorage;
use crate::PackageWithDeploymentToTest;
use crate::TestResult;
use crate::TestDetails;


/// Required test setup for package types that requires a deployment.
#[derive(Debug)]
pub enum DeploymentSetup {
    Script(ScriptTestSetup),
    Contract(ContractTestSetup),
}

impl DeploymentSetup {
    /// Returns the storage for this test setup
    fn storage(&self) -> &vm::storage::MemoryStorage {
        match self {
            DeploymentSetup::Script(script_setup) => &script_setup.storage,
            DeploymentSetup::Contract(contract_setup) => &contract_setup.storage,
        }
    }

    /// Return the root contract id if this is a contract setup.
    fn root_contract_id(&self) -> Option<tx::ContractId> {
        match self {
            DeploymentSetup::Script(_) => None,
            DeploymentSetup::Contract(contract_setup) => Some(contract_setup.root_contract_id),
        }
    }
}

/// The storage and the contract id (if a contract is being tested) for a test.
#[derive(Debug)]
pub enum TestSetup {
    WithDeployment(DeploymentSetup),
    WithoutDeployment(vm::storage::MemoryStorage),
}

impl TestSetup {
    /// Returns the storage for this test setup
    pub fn storage(&self) -> &vm::storage::MemoryStorage {
        match self {
            TestSetup::WithDeployment(deployment_setup) => deployment_setup.storage(),
            TestSetup::WithoutDeployment(storage) => storage,
        }
    }

    /// Produces an iterator yielding contract ids of contract dependencies for this test setup.
    pub fn contract_dependency_ids(&self) -> impl Iterator<Item = &tx::ContractId> + '_ {
        match self {
            TestSetup::WithDeployment(deployment_setup) => match deployment_setup {
                DeploymentSetup::Script(script_setup) => {
                    script_setup.contract_dependency_ids.iter()
                }
                DeploymentSetup::Contract(contract_setup) => {
                    contract_setup.contract_dependency_ids.iter()
                }
            },
            TestSetup::WithoutDeployment(_) => [].iter(),
        }
    }

    /// Return the root contract id if this is a contract setup.
    pub fn root_contract_id(&self) -> Option<tx::ContractId> {
        match self {
            TestSetup::WithDeployment(deployment_setup) => deployment_setup.root_contract_id(),
            TestSetup::WithoutDeployment(_) => None,
        }
    }

    /// Produces an iterator yielding all contract ids required to be included in the transaction
    /// for this test setup.
    pub fn contract_ids(&self) -> impl Iterator<Item = tx::ContractId> + '_ {
        self.contract_dependency_ids()
            .cloned()
            .chain(self.root_contract_id())
    }
}

/// The data collected to test a contract.
#[derive(Debug)]
pub struct ContractTestSetup {
    pub storage: vm::storage::MemoryStorage,
    pub contract_dependency_ids: Vec<tx::ContractId>,
    pub root_contract_id: tx::ContractId,
}

/// The data collected to test a script.
#[derive(Debug)]
pub struct ScriptTestSetup {
    pub storage: vm::storage::MemoryStorage,
    pub contract_dependency_ids: Vec<tx::ContractId>,
}
