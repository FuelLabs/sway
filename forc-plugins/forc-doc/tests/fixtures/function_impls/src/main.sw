library;

use std::option::Option;

/// The `Identity` type: either an `Address` or a `ContractId`.
pub enum Identity {
    /// An `Address` variant.
    Address: u32,
    /// A `ContractId` variant.
    ContractId: u32,
}

impl Identity {
    /// Returns the `Address` variant.
    pub fn as_address(self) -> Option<u32> {
        match self {
            Identity::Address(a) => Some(a),
            _ => None,
        }
    }

    /// Returns the `ContractId` variant.
    pub fn as_contract_id(self) -> Option<u32> {
        match self {
            Identity::ContractId(c) => Some(c),
            _ => None,
        }
    }

    /// A really amazing function that adds three numbers together.
    pub fn foo(arg1: u32, arg2: u32, arg3: u32) -> u32 {
        arg1 + arg2 + arg3
    }
}