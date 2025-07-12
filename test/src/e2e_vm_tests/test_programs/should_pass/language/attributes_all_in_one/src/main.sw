//! This is an all-in-one example that demonstrates all valid
//! usages of attributes.
contract;

// TODO: Extend with testing nested items once https://github.com/FuelLabs/sway/issues/6932 is implemented.

mod lib;

/// Comment.
/// Comment.
#[allow(dead_code, deprecated)]
#[cfg(target = "fuel")]
/// Comment.
#[cfg(program_type = "contract")]
#[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
use lib::*;

/// Comment.
/// Comment.
#[allow(dead_code)]
#[allow(deprecated)]
#[cfg(target = "fuel")]
/// Comment.
#[cfg(program_type = "contract")]
#[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
storage {
    /// Comment.
    /// Comment.
    #[allow(dead_code, deprecated)]
    #[cfg(target = "fuel")]
    /// Comment.
    #[cfg(program_type = "contract")]
    #[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
    x: u8 = 0,
    /// Comment.
    /// Comment.
    #[allow(dead_code, deprecated)]
    #[cfg(target = "fuel")]
    #[cfg(program_type = "contract")]
    /// Comment.
    #[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
    ns_1 {
        /// Comment.
        /// Comment.
        #[allow(dead_code)]
        #[allow(deprecated)]
        #[cfg(target = "fuel")]
        /// Comment.
        #[cfg(program_type = "contract")]
        #[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
        x: u8 = 0,
        /// Comment.
        /// Comment.
        #[allow(dead_code)]
        #[allow(deprecated)]
        #[cfg(target = "fuel")]
        /// Comment.
        #[cfg(program_type = "contract")]
        #[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
        ns_2 {
            /// Comment.
            /// Comment.
            #[allow(dead_code)]
            #[allow(deprecated)]
            #[cfg(target = "fuel")]
            #[cfg(program_type = "contract")]
            /// Comment.
            #[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
            x: u8 = 0,
        }
    }
}

/// Comment.
/// Comment.
#[allow(deprecated, dead_code)]
#[cfg(target = "fuel")]
/// Comment.
#[cfg(program_type = "contract")]
#[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
configurable {
    /// Comment.
    /// Comment.
    #[allow(dead_code)]
    #[allow(deprecated)]
    #[deprecated(note = "note")]
    #[cfg(target = "fuel")]
    /// Comment.
    #[cfg(program_type = "contract")]
    #[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
    X: u8 = 0,
}

/// Comment.
/// Comment.
#[allow(deprecated)]
#[allow(dead_code)]
/// Comment.
#[cfg(target = "fuel")]
#[cfg(program_type = "contract")]
#[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
abi Abi {
    /// Comment.
    /// Comment.
    #[storage(read)]
    #[payable]
    #[allow(deprecated)]
    #[allow(dead_code)]
    /// Comment.
    #[cfg(target = "fuel")]
    #[cfg(program_type = "contract")]
    #[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
    fn abi_function();
    /// Comment.
    /// Comment.
    #[allow(deprecated)]
    #[allow(dead_code)]
    /// Comment.
    #[cfg(target = "fuel")]
    #[cfg(program_type = "contract")]
    #[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
    const ABI_CONST: u8 = 0;
} {
    /// Comment.
    /// Comment.
    #[storage(read, write)]
    #[inline(always)]
    #[trace(always)]
    #[payable]
    #[allow(deprecated)]
    #[allow(dead_code)]
    #[deprecated(note = "note")]
    /// Comment.
    #[cfg(target = "fuel")]
    #[cfg(program_type = "contract")]
    #[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
    fn abi_provided_function() {
        let _ = 0;
        panic "Panics for tracing purposes.";
    }
}

/// Comment.
/// Comment.
#[allow(deprecated)]
#[allow(dead_code)]
/// Comment.
#[cfg(target = "fuel")]
#[cfg(program_type = "contract")]
#[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
impl Abi for Contract {
    /// Comment.
    /// Comment.
    #[storage(read)]
    #[inline(always)]
    #[trace(always)]
    #[payable]
    /// Comment.
    #[allow(deprecated, dead_code)]
    #[deprecated(note = "note")]
    #[cfg(target = "fuel")]
    #[cfg(program_type = "contract")]
    #[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
    fn abi_function() {
        let _ = 0;
        panic "Panics for tracing purposes.";
    }
    /// Comment.
    /// Comment.
    #[allow(deprecated, dead_code)]
    #[deprecated(note = "note")]
    /// Comment.
    #[cfg(target = "fuel")]
    #[cfg(program_type = "contract")]
    #[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
    const ABI_CONST: u8 = 0;
}

/// Comment.
/// Comment.
#[storage(read)]
#[inline(always)]
#[trace(always)]
#[allow(deprecated, dead_code)]
#[deprecated(note = "note")]
/// Comment.
#[cfg(target = "fuel")]
#[cfg(program_type = "contract")]
#[fallback]
#[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
fn fallback() {
    panic "Panics for tracing purposes.";
}

/// Comment.
/// Comment.
#[storage(read)]
#[inline(never)]
#[trace(never)]
#[allow(deprecated)]
#[allow(dead_code)]
#[deprecated(note = "note")]
/// Comment.
#[cfg(target = "fuel")]
#[cfg(program_type = "contract")]
#[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
fn module_function() {
    panic "Panics for tracing purposes.";
}

/// Comment.
/// Comment.
#[storage(read)]
#[inline(always)]
#[trace(always)]
#[test]
#[allow(deprecated)]
#[allow(dead_code)]
#[deprecated(note = "note")]
/// Comment.
#[cfg(target = "fuel")]
#[cfg(program_type = "contract")]
#[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
fn test_function() {
    panic "Panics for tracing purposes.";
}

/// Comment.
/// Comment.
#[storage(read)]
#[inline(always)]
#[trace(always)]
#[test(should_revert)]
#[allow(deprecated)]
#[allow(dead_code)]
#[deprecated(note = "note")]
/// Comment.
#[cfg(target = "fuel")]
#[cfg(program_type = "contract")]
#[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
fn test_function_should_revert() {
    panic "Panics for tracing purposes.";
}

/// Comment.
/// Comment.
#[storage(read)]
#[inline(always)]
#[trace(always)]
#[test(should_revert = "18446744073709486084")]
/// Comment.
#[allow(deprecated)]
#[allow(dead_code)]
#[deprecated(note = "note")]
#[cfg(target = "fuel")]
#[cfg(program_type = "contract")]
#[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
/// Comment.
fn test_function_should_revert_with_code() {
    panic "Panics for tracing purposes.";
}

/// Comment.
/// Comment.
#[allow(deprecated, dead_code)]
#[deprecated(note = "note")]
#[cfg(target = "fuel")]
#[cfg(program_type = "contract")]
#[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
/// Comment.
const MODULE_CONST: u8 = 0;
