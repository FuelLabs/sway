use serde::{Deserialize, Serialize};

use crate::constants::{DEFAULT_ENUM_VARIANT_ALIGN_THRESHOLD, DEFAULT_STRUCT_FIELD_ALIGN_THRESHOLD};

#[derive(Serialize, Deserialize, Debug)]
pub struct Structures {
    /// Align enum variants discrims, if their diffs fit within threshold.
    pub enum_discrim_align_threshold: usize,
    /// Align struct fields if their diffs fits within threshold.
    pub struct_field_align_threshold: usize,
    /// Put small struct literals on a single line.
    pub struct_lit_single_line: bool,
}

impl Default for Structures {
    fn default() -> Self {
        Self {
            enum_discrim_align_threshold: DEFAULT_ENUM_VARIANT_ALIGN_THRESHOLD,
            struct_field_align_threshold: DEFAULT_STRUCT_FIELD_ALIGN_THRESHOLD,
            struct_lit_single_line: true,
        }
    }
}
