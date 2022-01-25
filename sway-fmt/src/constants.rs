// patterns should be invalid Sway code
// they inform CodeBuilder (`second` pass) of the formatter what should it do
// if it comes across a known pattern
pub const NEW_LINE_PATTERN: &str = "+/+";
pub const ALREADY_FORMATTED_LINE_PATTERN: &str = "---";

// TODO there is a circular dep here between sway-server's definition of tab size and ours
// we should just standardize when swayfmt.toml gets implemented
pub const TAB_SIZE: usize = 4;
