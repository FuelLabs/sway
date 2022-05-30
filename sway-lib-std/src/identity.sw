//! A wrapper type with 2 variants; an `Address` and a `ContractId`
library identity;

use ::address::Address;
use ::contract_id::ContractId;

pub enum Identity {
    Address: Address,
    ContractId: ContractId,
}

// idea...
// pub trait Indentify() {
//     fn resolve
// }
