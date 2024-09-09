//! Defines the Sway standard library prelude.
//! The prelude consists of implicitly available items,
//! for which `use` is not required.
library;

// Collections
pub use ::vec::{Vec, VecIter};

// Error handling
pub use ::assert::{assert, assert_eq, assert_ne};
pub use ::option::Option::{self, *};
pub use ::result::Result::{self, *};
pub use ::revert::{require, revert};

// Convert
pub use ::convert::From;

/// U128
pub use ::u128::*;

// Primitive conversions
pub use ::primitive_conversions::{b256::*, str::*, u16::*, u256::*, u32::*, u64::*, u8::*,};

// Logging
pub use ::logging::log;

// Math
pub use ::math::*;
