//! This is a module comment that also becomes an attribute.
//! This line will also be an attribute.

//! Having an empty line is allowed.
library;

/// Comment.
/// Comment.
#[allow(deprecated, dead_code)]
#[cfg(target = "fuel")]
#[cfg(program_type = "contract")]
#[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
type MyU64 = u64;

/// Comment.
/// Comment.
#[allow(dead_code)]
#[allow(deprecated)]
#[cfg(target = "fuel")]
#[cfg(program_type = "contract")]
#[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
trait T {
    /// Comment.
    /// Comment.
    #[allow(dead_code)]
    #[allow(deprecated)]
    #[cfg(target = "fuel")]
    #[cfg(program_type = "contract")]
    #[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
    type Type;
    /// Comment.
    /// Comment.
    #[allow(dead_code)]
    #[allow(deprecated)]
    #[cfg(target = "fuel")]
    #[cfg(program_type = "contract")]
    #[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
    const TRAIT_CONST: u8 = 0;
    /// Comment.
    /// Comment.
    #[storage(read)]
    #[allow(dead_code)]
    #[allow(deprecated)]
    #[cfg(target = "fuel")]
    #[cfg(program_type = "contract")]
    #[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
    fn trait_assoc_fn();
    /// Comment.
    /// Comment.
    #[storage(read, write)]
    #[allow(dead_code)]
    #[allow(deprecated)]
    #[cfg(target = "fuel")]
    #[cfg(program_type = "contract")]
    #[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
    fn trait_method();
} {
    /// Comment.
    /// Comment.
    #[storage(read)]
    #[inline(always)]
    #[trace(always)]
    #[allow(dead_code, deprecated)]
    #[cfg(target = "fuel")]
    #[cfg(program_type = "contract")]
    #[deprecated(note = "note")]
    #[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
    fn trait_provided_fn() {
        panic "Panics for tracing purposes.";
    }
    /// Comment.
    /// Comment.
    #[storage(read, write)]
    #[inline(never)]
    #[trace(never)]
    #[allow(dead_code)]
    #[allow(deprecated)]
    #[deprecated()]
    #[cfg(target = "fuel")]
    #[cfg(program_type = "contract")]
    #[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
    fn trait_provided_method() {
        panic "Panics for tracing purposes.";
    }
}

#[deprecated(note = "note")]
#[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
struct DeprecatedStructWithNote {}

/// Comment.
/// Comment.
#[allow(dead_code, deprecated)]
#[cfg(target = "fuel")]
#[cfg(program_type = "contract")]
#[deprecated]
#[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
struct S {
    /// Comment.
    /// Comment.
    #[allow(dead_code, deprecated)]
    #[deprecated(note = "note")]
    #[cfg(target = "fuel")]
    #[cfg(program_type = "contract")]
    #[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
    field: u8,
}

/// Comment.
/// Comment.
#[allow(dead_code, deprecated)]
#[cfg(target = "fuel")]
#[cfg(program_type = "contract")]
#[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
impl S {
    /// Comment.
    /// Comment.
    #[allow(dead_code)]
    #[allow(deprecated)]
    #[deprecated(note = "note")]
    #[cfg(target = "fuel")]
    #[cfg(program_type = "contract")]
    #[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
    const ASSOC_CONST: u8 = 0;
    /// Comment.
    /// Comment.
    #[storage(read)]
    #[inline(never)]
    #[trace(never)]
    #[allow(dead_code)]
    #[allow(deprecated)]
    #[deprecated(note = "note")]
    #[cfg(target = "fuel")]
    #[cfg(program_type = "contract")]
    #[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
    fn assoc_fn() {
        panic "Panics for tracing purposes.";
    }
    /// Comment.
    /// Comment.
    #[storage(read)]
    #[inline(always)]
    #[trace(always)]
    #[allow(dead_code)]
    #[allow(deprecated)]
    #[deprecated(note = "note")]
    #[cfg(target = "fuel")]
    #[cfg(program_type = "contract")]
    #[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
    fn method(self) {
        panic "Panics for tracing purposes.";
    }
}

/// Comment.
/// Comment.
#[allow(dead_code)]
#[allow(deprecated)]
#[cfg(target = "fuel")]
#[cfg(program_type = "contract")]
#[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
impl T for S {
    /// Comment.
    /// Comment.
    #[allow(dead_code)]
    #[allow(deprecated)]
    #[cfg(target = "fuel")]
    #[cfg(program_type = "contract")]
    #[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
    type Type = u8;
    /// Comment.
    /// Comment.
    #[allow(dead_code)]
    #[allow(deprecated)]
    #[deprecated(note = "note")]
    #[cfg(target = "fuel")]
    #[cfg(program_type = "contract")]
    #[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
    const TRAIT_CONST: u8 = 0;
    /// Comment.
    /// Comment.
    #[storage(read)]
    #[inline(always)]
    #[trace(always)]
    #[allow(dead_code)]
    #[allow(deprecated)]
    #[deprecated(note = "note")]
    #[cfg(target = "fuel")]
    #[cfg(program_type = "contract")]
    #[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
    fn trait_assoc_fn() {
        panic "Panics for tracing purposes.";
    }
    /// Comment.
    /// Comment.
    #[storage(read, write)]
    #[inline(never)]
    #[trace(never)]
    #[allow(dead_code)]
    #[allow(deprecated)]
    #[deprecated(note = "note")]
    #[cfg(target = "fuel")]
    #[cfg(program_type = "contract")]
    #[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
    fn trait_method() {
        panic "Panics for tracing purposes.";
    }
}

/// Comment.
/// Comment.
#[allow(dead_code)]
#[allow(deprecated)]
#[deprecated(note = "note")]
#[cfg(target = "fuel")]
#[cfg(program_type = "contract")]
#[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
#[error_type]
enum E {
    /// Comment.
    /// Comment.
    #[allow(dead_code)]
    #[allow(deprecated)]
    #[deprecated(note = "note")]
    #[cfg(target = "fuel")]
    #[cfg(program_type = "contract")]
    #[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
    #[error(m = "msg")]
    A: (),
}

/// Comment.
/// Comment.
#[allow(dead_code)]
#[allow(deprecated)]
#[cfg(target = "fuel")]
#[cfg(program_type = "contract")]
#[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
impl E {
    /// Comment.
    /// Comment.
    #[allow(dead_code)]
    #[allow(deprecated)]
    #[deprecated(note = "note")]
    #[cfg(target = "fuel")]
    #[cfg(program_type = "contract")]
    #[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
    const ASSOC_CONST: u8 = 0;
    /// Comment.
    /// Comment.
    #[storage(read)]
    #[inline(never)]
    #[trace(never)]
    #[allow(dead_code)]
    #[allow(deprecated)]
    #[deprecated(note = "note")]
    #[cfg(target = "fuel")]
    #[cfg(program_type = "contract")]
    #[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
    fn assoc_fn() {
        panic "Panics for tracing purposes.";
    }
    /// Comment.
    /// Comment.
    #[storage(read, write)]
    #[inline(always)]
    #[trace(always)]
    #[allow(dead_code)]
    #[allow(deprecated)]
    #[deprecated(note = "note")]
    #[cfg(target = "fuel")]
    #[cfg(program_type = "contract")]
    #[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
    fn method(self) {
        panic "Panics for tracing purposes.";
    }
}

/// Comment.
/// Comment.
#[allow(dead_code)]
#[allow(deprecated)]
#[cfg(target = "fuel")]
#[cfg(program_type = "contract")]
#[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
impl T for E {
    /// Comment.
    /// Comment.
    #[allow(dead_code)]
    #[allow(deprecated)]
    #[cfg(target = "fuel")]
    #[cfg(program_type = "contract")]
    #[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
    type Type = u8;
    /// Comment.
    /// Comment.
    #[allow(dead_code)]
    #[allow(deprecated)]
    #[deprecated(note = "note")]
    #[cfg(target = "fuel")]
    #[cfg(program_type = "contract")]
    #[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
    const TRAIT_CONST: u8 = 0;
    /// Comment.
    /// Comment.
    #[storage(read)]
    #[inline(always)]
    #[trace(always)]
    #[allow(dead_code)]
    #[allow(deprecated)]
    #[deprecated(note = "note")]
    #[cfg(target = "fuel")]
    #[cfg(program_type = "contract")]
    #[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
    fn trait_assoc_fn() {
        panic "Panics for tracing purposes.";
    }
    /// Comment.
    /// Comment.
    #[storage(read, write)]
    #[inline(never)]
    #[trace(always)]
    #[allow(dead_code)]
    #[allow(deprecated)]
    #[deprecated(note = "note")]
    #[cfg(target = "fuel")]
    #[cfg(program_type = "contract")]
    #[unknown_0, unknown_1(arg), unknown_2(arg_1 = "value", arg_2)]
    fn trait_method() {
        panic "Panics for tracing purposes.";
    }
}
