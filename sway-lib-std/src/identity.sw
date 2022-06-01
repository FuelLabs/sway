//! A wrapper type with 2 variants (`Address` or `ContractId`) sed to represent either in a generic way. The use of this type allows for handling interactions with contracts and addresses in a consistent manner.
library identity;

use ::address::Address;
use ::contract_id::ContractId;

pub enum Identity {
    Address: Address,
    ContractId: ContractId,
}
