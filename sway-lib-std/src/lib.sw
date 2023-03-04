//! The official standard library for the Sway smart contract language.
//!
//! Source: https://github.com/FuelLabs/sway/tree/master/sway-lib-std
library std;

dep error_signals;
dep logging;
dep revert;
dep result;
dep option;
dep convert;
dep intrinsics;
dep assert;
dep alloc;
dep contract_id;
dep constants;
dep external;
dep registers;
dep call_frames;
dep context;
dep hash;
dep b512;
dep address;
dep identity;
dep vec;
dep bytes;
dep r#storage;
dep b256;
dep tx;
dep inputs;
dep outputs;
dep auth;
dep math;
dep block;
dep token;
dep ecr;
dep vm/mod;
dep flags;
dep u128;
dep u256;
dep message;
dep prelude;
dep low_level_call;

use core::*;
