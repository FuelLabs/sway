//! Defines the Sway standard library prelude.
//! The prelude consists of implicitly available items,
//! for which `use` is not required.
library;

// Collections
use ::vec::{Vec, VecIter};

// Error handling
use ::assert::{assert, assert_eq, assert_ne};
use ::option::Option::{self, *};
use ::result::Result::{self, *};
use ::revert::{require, revert};

// Convert
use ::convert::From;

// Logging
use ::logging::log;
