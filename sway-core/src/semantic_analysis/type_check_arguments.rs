use crate::build_config::BuildConfig;
use crate::control_flow_analysis::ControlFlowGraph;
use crate::parse_tree::declaration::Purity;
use crate::semantic_analysis::{ast_node::Mode, *};
use crate::type_engine::*;

pub struct TypeCheckArguments<'a, T> {
    pub checkee: T,
    /// An immutable namespace that consists of the names that should always be present, no matter
    /// what module or scope we are currently checking.
    ///
    /// These include external library dependencies and (when it's added) the `std` prelude.
    ///
    /// This is passed through type-checking in order to initialise the namespace of each submodule
    /// within the project.
    pub init: &'a namespace::Module,
    /// Access to the `root` of the project namespace.
    ///
    /// From the root, the entirety of the project's namespace can always be accessed.
    ///
    /// The root is initialised from the `global` namespace.
    pub root: &'a mut namespace::Root,
    /// An absolute path from the `root` that represents the current module being checked.
    pub mod_path: &'a namespace::Path,
    pub return_type_annotation: TypeId,
    pub help_text: &'static str,
    pub self_type: TypeId,
    pub build_config: &'a BuildConfig,
    pub dead_code_graph: &'a mut ControlFlowGraph,
    pub mode: Mode,
    pub opts: TCOpts,
}

#[derive(Default, Clone, Copy)]
pub struct TCOpts {
    pub(crate) purity: Purity,
}
