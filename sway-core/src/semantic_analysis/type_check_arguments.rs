use crate::build_config::BuildConfig;
use crate::control_flow_analysis::ControlFlowGraph;
use crate::parse_tree::declaration::Purity;
use crate::semantic_analysis::{ast_node::Mode, *};
use crate::type_engine::*;

pub struct TypeCheckArguments<'a, T> {
    pub checkee: T,
    pub namespace: NamespaceRef,
    pub crate_namespace: NamespaceRef,
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
