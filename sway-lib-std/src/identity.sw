//! A wrapper type with 2 variants, `Address` and `ContractId`. The use of this type allows for handling interactions with contracts and addresses in a unified manner.
library identity;

use ::address::Address;
use ::contract_id::ContractId;
use ::intrinsics::size_of_val;
use ::mem::{addr_of, eq};

pub enum Identity {
    Address: Address,
    ContractId: ContractId,
}

impl core::ops::Eq for Identity {
    fn eq(self, other: Self) -> bool {
        match(self, other) {
            (Identity::Address(address1), Identity::Address(address2)) => eq(addr_of(address1), addr_of(address2), size_of_val(self)), (Identity::ContractId(asset1), Identity::ContractId(asset2)) => eq(addr_of(asset1), addr_of(asset2), size_of_val(self)), _ => false, 
        }
    }
}
