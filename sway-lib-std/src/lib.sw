//! The official standard library for the Sway smart contract language.
//!
//! Source: https://github.com/FuelLabs/sway/tree/master/sway-lib-std
library;

mod error_signals;
mod logging;
mod revert;
mod result;
mod option;
mod convert;
mod intrinsics;
mod assert;
mod alloc;
mod contract_id;
mod constants;
mod external;
mod registers;
mod call_frames;
mod context;
mod hash;
mod b512;
mod address;
mod identity;
mod vec;
mod bytes;
mod r#storage;
mod experimental;
mod b256;
mod tx;
mod inputs;
mod outputs;
mod auth;
mod math;
mod block;
mod token;
mod ecr;
mod vm;
mod flags;
mod u128;
mod u256;
mod message;
mod prelude;
mod low_level_call;

use core::*;
