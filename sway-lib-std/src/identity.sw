//! A wrapper type with 2 variants, `Address` and `ContractId`. The use of this type allows for handling interactions with contracts and addresses in a unified manner.
library identity;

use ::address::Address;
use ::contract_id::ContractId;

pub enum Identity {
    Address: Address,
    ContractId: ContractId,
}
