use crate::build_config::BuildConfig;
use crate::control_flow_analysis::ControlFlowGraph;


use crate::semantic_analysis::{ast_node::Mode, Namespace};


use crate::type_engine::*;



use std::collections::{HashMap, HashSet};
pub struct TypeCheckArguments<'a, 'sc, T> {
    pub(crate) checkee: T,
    pub(crate) namespace: &'a mut Namespace<'sc>,
    pub(crate) crate_namespace: Option<&'a Namespace<'sc>>,
    pub(crate) return_type_annotation: TypeId,
    pub(crate) help_text: &'static str,
    pub(crate) self_type: TypeId,
    pub(crate) build_config: &'a BuildConfig,
    pub(crate) dead_code_graph: &'a mut ControlFlowGraph<'sc>,
    pub(crate) mode: Mode,
    pub(crate) dependency_graph: &'a mut HashMap<String, HashSet<String>>,
}
