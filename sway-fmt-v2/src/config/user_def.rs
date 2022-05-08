use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Structures {
    /// Align enum variants discrims, if their diffs fit within threshold.
    pub enum_discrim_align_threshold: usize,
    /// Align struct fields if their diffs fits within threshold.
    pub struct_field_align_threshold: usize,
    /// Put small struct literals on a single line.
    pub struct_lit_single_line: bool,
}
