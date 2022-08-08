use sway_core::TreeType;
use tower_lsp::lsp_types::Range;

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum RunnableType {
    /// This is the main_fn entry point for the predicate or script.
    MainFn,
    /// Place holder for when we have in language testing supported.
    /// The field holds the index of the test to run.
    _TestFn(u8),
}

#[derive(Debug, Eq, PartialEq)]
pub struct Runnable {
    /// The location in the file where the runnable button should be displayed
    pub range: Range,
    /// The program kind of the current file
    pub tree_type: TreeType,
}

impl Runnable {
    pub fn new(range: Range, tree_type: TreeType) -> Self {
        Self { range, tree_type }
    }
}
