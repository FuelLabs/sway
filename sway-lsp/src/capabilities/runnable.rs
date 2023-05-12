use serde_json::{json, Value};
use sway_core::language::parsed::TreeType;
use sway_types::Span;
use tower_lsp::lsp_types::{Command, Range};

use crate::core::token::get_range_from_span;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct RunnableMainFn {
    /// The location in the file where the runnable button should be displayed
    pub span: Span,
    /// The program kind of the current file
    pub tree_type: TreeType,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct RunnableTestFn {
    /// The location in the file where the runnable button should be displayed.
    pub span: Span,
    /// The program kind of the current file.
    pub tree_type: TreeType,
    /// Additional arguments to use with the runnable command.
    pub test_name: Option<String>,
}

/// A runnable is a sway function that can be executed in the editor.
pub trait Runnable: core::fmt::Debug + Send + Sync + 'static {
    /// The command to execute.
    fn command(&self) -> Command {
        Command {
            command: self.cmd_string(),
            title: self.label_string(),
            arguments: self.arguments(),
        }
    }
    /// The command name defined in the client.
    fn cmd_string(&self) -> String;
    /// The label to display in the editor.
    fn label_string(&self) -> String;
    /// The arguments to pass to the command.
    fn arguments(&self) -> Option<Vec<Value>>;
    /// The span where the runnable button should be displayed.
    fn span(&self) -> &Span;
    /// The range in the file where the runnable button should be displayed.
    fn range(&self) -> Range {
        get_range_from_span(self.span())
    }
}

impl Runnable for RunnableMainFn {
    fn cmd_string(&self) -> String {
        "sway.runScript".to_string()
    }
    fn label_string(&self) -> String {
        "▶\u{fe0e} Run".to_string()
    }
    fn arguments(&self) -> Option<Vec<Value>> {
        None
    }
    fn span(&self) -> &Span {
        &self.span
    }
}

impl Runnable for RunnableTestFn {
    fn cmd_string(&self) -> String {
        "sway.runTests".to_string()
    }
    fn label_string(&self) -> String {
        "▶\u{fe0e} Run Test".to_string()
    }
    fn arguments(&self) -> Option<Vec<Value>> {
        self.test_name
            .as_ref()
            .map(|test_name| vec![json!({ "name": test_name })])
    }
    fn span(&self) -> &Span {
        &self.span
    }
}
