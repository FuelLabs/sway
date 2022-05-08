use serde::{Deserialize, Serialize};
use config::options::*;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct SwayFormatConfig {
    // TODO: make default 100
    max_width: usize, // Max width of each line
    // TODO: make default false
    hard_tabs: bool, // Use tab characters for indentation, spaces for alignment
    // TODO: make default 4
    tab_spaces: usize, // Number of spaces per tab
    // TODO: make default Auto
    newline_style: NewlineStyle, // Unix or Windows line endings
    // TODO: make default Block
    indent_style: IndentStyle, // How we indent expressions or items
}
