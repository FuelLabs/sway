use crate::error::{Error, Result};
use dap::types::Breakpoint;
use fuel_types::ContractId;
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    path::PathBuf,
};
use sway_core::asm_generation::ProgramABI;

pub type ExitCode = i64;
pub type Instruction = u64;
pub type Breakpoints = HashMap<PathBuf, Vec<Breakpoint>>;

/// A map storing ABIs for contracts, capable of fetching ABIs from the registry for unknown contracts.
#[derive(Debug, Default)]
pub struct AbiMap(HashMap<ContractId, ProgramABI>);

impl AbiMap {
    /// Registers the given ABI for the given contract ID.
    pub fn register_abi(&mut self, contract_id: ContractId, abi: ProgramABI) {
        self.insert(contract_id, abi);
    }

    /// Either fetches the ABI from the Sway ABI Registry or returns it from the cache if it's already known.
    pub fn get_or_fetch_abi(&mut self, contract_id: &ContractId) -> Option<&ProgramABI> {
        // If we already have it, return it
        if self.contains_key(contract_id) {
            return self.get(contract_id);
        }

        // Try to fetch from ABI Registry
        match fetch_abi_from_registry(contract_id) {
            Ok(abi) => {
                self.register_abi(*contract_id, abi);
                self.get(contract_id)
            }
            Err(_) => None,
        }
    }
}

impl Deref for AbiMap {
    type Target = HashMap<ContractId, ProgramABI>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AbiMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Fetches the ABI for the given contract ID from the Sway ABI Registry.
fn fetch_abi_from_registry(_contract_id: &ContractId) -> Result<ProgramABI> {
    // TODO: Implement this once the Sway ABI Registry is available
    // See this github issue: https://github.com/FuelLabs/sway/issues/6862
    Err(Error::AbiError("Not implemented yet".to_string()))
}
