//! Configuration options related to formatting user-defined structures.

use crate::constants::{
    DEFAULT_ENUM_VARIANT_ALIGN_THRESHOLD, DEFAULT_STORAGE_FIELD_ALIGN_THRESHOLD,
    DEFAULT_STRUCT_FIELD_ALIGN_THRESHOLD,
};

use super::user_opts::StructuresOptions;

/// Styling preferences for user-defined structures like `struct`s or `enum`s.
#[derive(Debug)]
pub struct Structures {
    /// Align enum variants discrims, if their diffs fit within threshold.
    pub enum_variant_align_threshold: usize,
    /// Align struct fields if their diffs fits within threshold.
    pub struct_field_align_threshold: usize,
    /// Align storage fields if their diffs fit within the threshold.
    pub storage_field_align_threshold: usize,
    /// Put small struct literals on a single line.
    pub struct_lit_single_line: bool,
}

impl Default for Structures {
    fn default() -> Self {
        Self {
            enum_variant_align_threshold: DEFAULT_ENUM_VARIANT_ALIGN_THRESHOLD,
            struct_field_align_threshold: DEFAULT_STRUCT_FIELD_ALIGN_THRESHOLD,
            storage_field_align_threshold: DEFAULT_STORAGE_FIELD_ALIGN_THRESHOLD,
            struct_lit_single_line: true,
        }
    }
}

impl Structures {
    pub fn from_opts(opts: &StructuresOptions) -> Self {
        let default = Self::default();
        Self {
            enum_variant_align_threshold: opts
                .enum_variant_align_threshold
                .unwrap_or(default.enum_variant_align_threshold),
            struct_field_align_threshold: opts
                .struct_field_align_threshold
                .unwrap_or(default.struct_field_align_threshold),
            storage_field_align_threshold: opts
                .storage_field_align_threshold
                .unwrap_or(default.storage_field_align_threshold),
            struct_lit_single_line: opts
                .struct_lit_single_line
                .unwrap_or(default.struct_lit_single_line),
        }
    }
}
