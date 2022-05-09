//! Configuration options related to formatting literals.
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Literals {
    /// Format string literals where necessary.
    pub format_strings: bool,
    /// Format hexadecimal integer literals.
    pub hex_literal_case: HexLiteralCase,
}

impl Default for Literals {
    fn default() -> Self {
        Self {
            format_strings: false,
            hex_literal_case: HexLiteralCase::Preserve,
        }
    }
}

/// Controls how swayfmt should handle case in hexadecimal literals.
#[derive(Serialize, Deserialize, Debug)]
pub enum HexLiteralCase {
    /// Leave the literal as-is
    Preserve,
    /// Ensure all literals use uppercase lettering
    Upper,
    /// Ensure all literals use lowercase lettering
    Lower,
}
