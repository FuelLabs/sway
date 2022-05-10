//! Configuration options related to formatting user-defined structures.

use crate::constants::{
    DEFAULT_ENUM_VARIANT_ALIGN_THRESHOLD, DEFAULT_STRUCT_FIELD_ALIGN_THRESHOLD,
};

/// Styling preferences for user-defined structures like `struct`s or `enum`s.
#[derive(Debug)]
pub struct Structures {
    /// Align enum variants discrims, if their diffs fit within threshold.
    pub enum_variant_align_threshold: usize,
    /// Align struct fields if their diffs fits within threshold.
    pub struct_field_align_threshold: usize,
    /// Put small struct literals on a single line.
    pub struct_lit_single_line: bool,
}

impl Default for Structures {
    fn default() -> Self {
        Self {
            enum_variant_align_threshold: DEFAULT_ENUM_VARIANT_ALIGN_THRESHOLD,
            struct_field_align_threshold: DEFAULT_STRUCT_FIELD_ALIGN_THRESHOLD,
            struct_lit_single_line: true,
        }
    }
}
