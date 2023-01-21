use serde_json::Value;
use sway_core::language::parsed::TreeType;
use tower_lsp::lsp_types::{Command, Range};

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum RunnableKind {
    /// This is the main_fn entry point for the predicate or script.
    MainFn,
    /// Place holder for when we have in language testing supported.
    /// The field holds the index of the test to run.
    TestFn(u8),
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Runnable {
    /// The kind of runnable
    pub kind: RunnableKind,
    /// The location in the file where the runnable button should be displayed
    pub range: Range,
    /// The program kind of the current file
    pub tree_type: TreeType,
    /// Additional arguments to use with the runnable command
    pub arguments: Option<Vec<Value>>,
}

impl Runnable {
    pub fn new(
        kind: RunnableKind,
        range: Range,
        tree_type: TreeType,
        arguments: Option<Vec<Value>>,
    ) -> Self {
        Self {
            kind,
            range,
            tree_type,
            arguments,
        }
    }

    pub(crate) fn command(&self) -> Command {
        Command {
            command: self.cmd_string(),
            title: self.label_string(),
            arguments: self.arguments.clone(),
        }
    }

    fn cmd_string(&self) -> String {
        match self.kind {
            RunnableKind::MainFn => "sway.runScript".to_string(),
            RunnableKind::TestFn(_) => "sway.runTests".to_string(),
        }
    }

    fn label_string(&self) -> String {
        match self.kind {
            RunnableKind::MainFn => "▶\u{fe0e} Run".to_string(),
            RunnableKind::TestFn(_) => "▶\u{fe0e} Run Test".to_string(),
        }
    }
}
