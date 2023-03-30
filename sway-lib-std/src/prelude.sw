//! Defines the Sway standard library prelude.
//! The prelude consists of implicitly available items,
//! for which `use` is not required.
library;

// Blockchain types
use ::address::Address;
use ::contract_id::{ContractId, AssetId};
use ::identity::Identity;

// Collections
use ::storage::StorageMap;
use ::vec::Vec;

// Error handling
use ::assert::assert;
use ::option::Option;
use ::result::Result;
use ::revert::{require, revert};

// Convert
use ::convert::From;

// Logging
use ::logging::log;
