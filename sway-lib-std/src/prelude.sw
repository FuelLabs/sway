library prelude;

//! Defines the Sway standard library prelude.
//! The prelude consists of implicitly available items,
//! for which `use` is not required.

use ::address::Address;
use ::contract_id::ContractId;
use ::identity::Identity;
use ::vec::Vec;
use ::assert::assert;
use ::revert::require;
use ::revert::revert;
