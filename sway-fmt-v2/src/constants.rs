/// `swayfmt` file name.
pub const SWAY_FORMAT_FILE_NAME: &str = "swayfmt.toml";

//FUNDAMENTALS

/// Default max width of each line.
pub const DEFAULT_MAX_LINE_WIDTH: usize = 100;
/// Default tab size as spaces.
pub const DEFAULT_TAB_SPACES: usize = 4;

//HEURISTICS

/// Default max width of the args of a function call before falling back to vertical formatting.
pub const DEFAULT_FN_CALL_WIDTH: usize = 60;
/// Default max width of the args of a function-like attributes before falling back to vertical formatting.
pub const DEFAULT_ATTR_FN_LIKE_WIDTH: usize = 70;
/// Default max width in the body of a user-defined structure literal before falling back to vertical formatting.
pub const DEFAULT_STRUCTURE_LIT_WIDTH: usize = 18;
/// Default max width of a user-defined structure field before falling back to vertical formatting.
pub const DEFAULT_STRUCTURE_VAR_WIDTH: usize = 35;
/// Default Maximum width of an array literal before falling back to vertical formatting.
pub const DEFAULT_COLLECTION_WIDTH: usize = 60;
/// Defalt width threshold for an array element to be considered short.
pub const DEFAULT_SHORT_ARRAY_ELEM_WIDTH_THRESHOLD: usize = 10;
/// Default max length of a chain to fit on a single line.
pub const DEFAULT_CHAIN_WIDTH: usize = 60;
/// Default max line length for single line if-else expression.
pub const DEFAULT_SINGLE_LINE_IF_ELSE_WIDTH: usize = 50;

//ITEMS

/// Default max number of blank lines which can be put between items.
pub const DEFAULT_BLANK_LINES_UPPER_BOUND: usize = 1;
/// Default min number of blank lines which must be put between items.
pub const DEFAULT_BLANK_LINES_LOWER_BOUND: usize = 0;
/// Write an items and its attribute on the same line if their combined width is below a threshold.
pub const DEFAULT_INLINE_ATTR_WIDTH: usize = 0;

//COMMENTS

/// Default max length of comments.
pub const DEFAULT_MAX_COMMENT_WIDTH: usize = 80;

//NEWLINE_STYLE

pub(crate) const LINE_FEED: char = '\n';
pub(crate) const CARRIAGE_RETURN: char = '\r';
pub(crate) const WINDOWS_NEWLINE: &str = "\r\n";
pub(crate) const UNIX_NEWLINE: &str = "\n";

//INDENT_STYLE

// INDENT_BUFFER.len() = 81
pub(crate) const INDENT_BUFFER_LEN: usize = 80;
pub(crate) const INDENT_BUFFER: &str =
    "\n                                                                                ";
// 8096 is close enough to infinite according to `rustfmt`.
pub(crate) const INFINITE_SHAPE_WIDTH: usize = 8096;
pub(crate) const HARD_TAB: char = '\t';

/// Default max number of newlines allowed in between statements before collapsing them to
/// threshold
pub const DEFAULT_NEWLINE_THRESHOLD: usize = 1;

//IDENT
pub(crate) const RAW_MODIFIER: &str = "r#";
