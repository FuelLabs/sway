use lspower::lsp::{Position, Range};

pub(crate) fn make_range_end_inclusive(range: Range) -> Range {
    Range {
        start: range.start,
        end: Position {
            line: range.end.line,
            character: range.end.character + 1,
        },
    }
}
