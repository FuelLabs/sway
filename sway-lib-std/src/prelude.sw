//! Defines the Sway standard library prelude.
//! The prelude consists of implicitly available items,
//! for which `use` is not required.
library;

// Blockchain types
use ::address::Address;
use ::alias::SubId;
use ::asset_id::AssetId;
use ::contract_id::ContractId;
use ::identity::Identity;

// `StorageKey` API
use ::storage::storage_key::*;

// Iterator
// use ::iterator::*;

// Collections
use ::storage::storage_map::*;
use ::vec::{Vec, VecIter};

// Error handling
use ::assert::{assert, assert_eq, assert_ne};
use ::option::Option::{self, *};
use ::result::Result::{self, *};
use ::revert::{require, revert};

// Convert
use ::convert::From;

// Primitive conversions
use ::primitive_conversions::*;

// Logging
use ::logging::log;

// Auth
use ::auth::msg_sender;
