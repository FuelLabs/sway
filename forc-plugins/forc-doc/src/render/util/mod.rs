//! Utilities for managing edge cases in rendering types and their corresponding documentation.
pub mod format;

/// Strip the generic suffix from a type name. For example, `Foo<T>` would become `Foo`.
pub fn strip_generic_suffix(input: &str) -> &str {
    input.split_once('<').map_or(input, |(head, _)| head)
}
