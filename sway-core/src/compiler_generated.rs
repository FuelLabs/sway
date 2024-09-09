//! This module encapsulates generation of various elements generated internally by the compiler,
//! e.g., unique names of variables in desugared code and similar.
//! It also provides functions for inspecting such generated elements.

/// The prefix for the compiler generated names of tuples.
const TUPLE_VAR_NAME_PREFIX: &str = "__tuple_";

pub(crate) fn generate_tuple_var_name(suffix: usize) -> String {
    format!("{TUPLE_VAR_NAME_PREFIX}{suffix}")
}

pub fn is_generated_tuple_var_name(name: &str) -> bool {
    name.starts_with(TUPLE_VAR_NAME_PREFIX)
}

/// The prefix for the compiler generated names of structs used in destructuring
/// structs in `let` statements.
const DESTRUCTURED_STRUCT_VAR_NAME_PREFIX: &str = "__destructured_struct_";

pub(crate) fn generate_destructured_struct_var_name(suffix: usize) -> String {
    format!("{DESTRUCTURED_STRUCT_VAR_NAME_PREFIX}{suffix}")
}

pub fn is_generated_destructured_struct_var_name(name: &str) -> bool {
    name.starts_with(DESTRUCTURED_STRUCT_VAR_NAME_PREFIX)
}

/// The prefix for the compiler generated names of
/// variables that store values matched in match expressions.
const MATCHED_VALUE_VAR_NAME_PREFIX: &str = "__matched_value_";

pub(crate) fn generate_matched_value_var_name(suffix: usize) -> String {
    format!("{MATCHED_VALUE_VAR_NAME_PREFIX}{suffix}")
}

/// The prefix for the compiler generated names of
/// variables that store 1-based index of the OR match
/// alternative that gets matched, or zero if non of the
/// OR alternatives get matched.
const MATCHED_OR_VARIANT_INDEX_VAR_NAME_PREFIX: &str = "__matched_or_variant_index_";

pub(crate) fn generate_matched_or_variant_index_var_name(suffix: usize) -> String {
    format!("{MATCHED_OR_VARIANT_INDEX_VAR_NAME_PREFIX}{suffix}")
}

/// The prefix for the compiler generated names of
/// tuple variables that store values of the variables declared
/// in OR match alternatives.
const MATCHED_OR_VARIANT_VARIABLES_VAR_NAME_PREFIX: &str = "__matched_or_variant_variables_";

pub(crate) fn generate_matched_or_variant_variables_var_name(suffix: usize) -> String {
    format!("{MATCHED_OR_VARIANT_VARIABLES_VAR_NAME_PREFIX}{suffix}")
}

pub fn is_generated_any_match_expression_var_name(name: &str) -> bool {
    name.starts_with(MATCHED_VALUE_VAR_NAME_PREFIX)
        || name.starts_with(MATCHED_OR_VARIANT_INDEX_VAR_NAME_PREFIX)
        || name.starts_with(MATCHED_OR_VARIANT_VARIABLES_VAR_NAME_PREFIX)
}

/// A revert with this value signals that it was caused by an internal compiler error that
/// occurred during the flattening of match arms that contain variables in OR match patterns.
///
/// The value is: 14757395258967588865
pub(crate) const INVALID_MATCHED_OR_VARIABLE_INDEX_SIGNAL: u64 = 0xcccc_cccc_cccc_0001;

/// A revert with this value signals that it was caused by an internal compiler error that
/// occurred during the flattening of match arms that contain variables in OR match patterns.
///
/// The value is: 14757395258967588866
pub(crate) const INVALID_DESUGARED_MATCHED_EXPRESSION_SIGNAL: u64 = 0xcccc_cccc_cccc_0002;
