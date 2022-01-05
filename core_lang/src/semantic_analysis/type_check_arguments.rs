use crate::build_config::BuildConfig;
use crate::control_flow_analysis::ControlFlowGraph;
use crate::parse_tree::declaration::Purity;
use crate::semantic_analysis::{ast_node::Mode, Namespace};
use crate::type_engine::*;

use std::collections::{HashMap, HashSet};
pub struct TypeCheckArguments<'a, T> {
    pub(crate) checkee: T,
    pub(crate) namespace: &'a mut Namespace,
    pub(crate) crate_namespace: Option<&'a Namespace>,
    pub(crate) return_type_annotation: TypeId,
    pub(crate) help_text: &'static str,
    pub(crate) self_type: TypeId,
    pub(crate) build_config: &'a BuildConfig,
    pub(crate) dead_code_graph: &'a mut ControlFlowGraph,
    pub(crate) mode: Mode,
    pub(crate) dependency_graph: &'a mut HashMap<String, HashSet<String>>,
    pub(crate) opts: TCOpts,
}

#[derive(Default, Clone, Copy)]
pub struct TCOpts {
    pub(crate) purity: Purity,
}
