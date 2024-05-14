//! Defines the Sway standard library prelude.
//! The prelude consists of implicitly available items,
//! for which `use` is not required.
library;

// Error handling
use ::assert::{assert, assert_eq, assert_ne};
use ::revert::{require, revert};

// Logging
use ::logging::log;
