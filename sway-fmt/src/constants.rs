// patterns should be invalid Sway code
// they inform CodeBuilder (`second` pass) of the formatter what should it do
// if it comes across a known pattern
pub const NEW_LINE_PATTERN: &str = "+/+";
pub const ALREADY_FORMATTED_LINE_PATTERN: &str = "---";
