//! Configuration options related to formatting literals.
use crate::config::user_opts::LiteralsOptions;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Default)]
pub struct Literals {
    /// Format string literals where necessary.
    pub format_strings: bool,
    /// Format hexadecimal integer literals.
    pub hex_literal_case: HexLiteralCase,
}

impl Literals {
    pub fn from_opts(opts: &LiteralsOptions) -> Self {
        let default = Self::default();
        Self {
            format_strings: opts.format_strings.unwrap_or(default.format_strings),
            hex_literal_case: opts.hex_literal_case.unwrap_or(default.hex_literal_case),
        }
    }
}

/// Controls how swayfmt should handle case in hexadecimal literals.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
pub enum HexLiteralCase {
    /// Leave the literal as-is
    #[default]
    Preserve,
    /// Ensure all literals use uppercase lettering
    Upper,
    /// Ensure all literals use lowercase lettering
    Lower,
}
