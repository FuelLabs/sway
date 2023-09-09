//! Defines the Sway standard library prelude.
//! The prelude consists of implicitly available items,
//! for which `use` is not required.
library;

// Blockchain types
use ::address::Address;
use ::alias::SubId;
use ::contract_id::{AssetId, ContractId};
use ::identity::Identity;

// `StorageKey` API
use ::storage::storage_key::*;

// Collections
use ::storage::storage_map::*;
use ::vec::Vec;

// Error handling
use ::assert::{assert, assert_eq};
use ::option::Option::{*, self};
use ::result::Result::{*, self};
use ::revert::{require, revert};

// Convert
use ::convert::From;

// Primitive conversions
use ::primitive_conversions::*;

// Logging
use ::logging::log;

// Auth
use ::auth::msg_sender;
