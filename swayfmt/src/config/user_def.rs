//! Configuration options related to formatting user-defined structures.
use crate::config::user_opts::StructuresOptions;
use serde::{Deserialize, Serialize};

/// Styling preferences for user-defined structures like `struct`s or `enum`s.
#[derive(Debug, Clone, Copy)]
pub struct Structures {
    /// Align fields of user-defined structures if their diffs fit within threshold.
    pub field_alignment: FieldAlignment,
    /// Put small user-defined structure literals on a single line.
    pub small_structures_single_line: bool,
}

impl Default for Structures {
    fn default() -> Self {
        Self {
            field_alignment: FieldAlignment::Off,
            small_structures_single_line: true,
        }
    }
}

impl Structures {
    pub fn from_opts(opts: &StructuresOptions) -> Self {
        let default = Self::default();
        Self {
            field_alignment: opts.field_alignment.unwrap_or(default.field_alignment),
            small_structures_single_line: opts
                .struct_lit_single_line
                .unwrap_or(default.small_structures_single_line),
        }
    }
}

/// Align fields if they fit within a provided threshold.
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum FieldAlignment {
    AlignFields(usize),
    Off,
}
