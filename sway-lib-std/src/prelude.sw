//! Defines the Sway standard library prelude.
//! The prelude consists of implicitly available items,
//! for which `use` is not required.
library;

// Blockchain types
pub use ::address::Address;
pub use ::alias::SubId;
pub use ::asset_id::AssetId;
pub use ::contract_id::ContractId;
pub use ::identity::Identity;

// `StorageKey` API
pub use ::storage::storage_key::*;

// Collections
pub use ::storage::storage_map::*;
pub use ::vec::{Vec, VecIter};

// Error handling
pub use ::assert::{assert, assert_eq, assert_ne};
pub use ::option::Option::{self, *};
pub use ::result::Result::{self, *};
pub use ::revert::{require, revert};

// Convert
pub use ::convert::From;

// Primitive conversions
pub use ::primitive_conversions::{b256::*, str::*, u16::*, u256::*, u32::*, u64::*, u8::*,};

// Logging
pub use ::logging::log;

// Auth
pub use ::auth::msg_sender;

// Math
pub use ::math::*;
