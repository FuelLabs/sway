use std::collections::{HashMap, HashSet};

use crate::control_flow_analysis::ControlFlowGraph;
use crate::type_engine::TypeId;
use crate::{BuildConfig, CompileResult, MatchCondition, Namespace, TypeParameter};

use super::TypedScrutinee;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) enum TypedMatchCondition<'sc> {
    CatchAll,
    Scrutinee(TypedScrutinee<'sc>),
}

impl<'sc> TypedMatchCondition<'sc> {
    pub(crate) fn type_check(
        other: MatchCondition<'sc>,
        namespace: &mut Namespace<'sc>,
        type_annotation: Option<TypeId>,
        help_text: impl Into<String> + Clone,
        self_type: TypeId,
        build_config: &BuildConfig,
        dead_code_graph: &mut ControlFlowGraph<'sc>,
        dependency_graph: &mut HashMap<String, HashSet<String>>,
    ) -> CompileResult<'sc, Self> {
        unimplemented!()
    }

    /// Makes a fresh copy of all type ids in this expression. Used when monomorphizing.
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        unimplemented!()
    }
}
