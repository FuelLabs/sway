use std::collections::{HashMap, HashSet};

use super::*;
use crate::control_flow_analysis::ControlFlowGraph;
use crate::type_engine::TypeId;
use crate::{BuildConfig, CompileResult, MatchBranch, Namespace, TypeParameter};

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) struct TypedMatchBranch<'sc> {
    pub condition: TypedMatchCondition<'sc>,
    pub result: TypedExpression<'sc>,
}

impl<'sc> TypedMatchBranch<'sc> {
    pub(crate) fn type_check(
        other: MatchBranch<'sc>,
        primary_expression_type: Option<TypeId>,
        namespace: &mut Namespace<'sc>,
        type_annotation: Option<TypeId>,
        help_text: impl Into<String> + Clone,
        self_type: TypeId,
        build_config: &BuildConfig,
        dead_code_graph: &mut ControlFlowGraph<'sc>,
        dependency_graph: &mut HashMap<String, HashSet<String>>,
    ) -> CompileResult<'sc, Self> {
        /*
        let mut warnings = vec![];
        let mut errors = vec![];
        let condition = other.condition;
        let result = other.result;
        let typed_condition = check!(
            TypedMatchCondition::type_check(
                condition.clone(),
                namespace,
                Some(primary_expression.return_type),
                "",
                self_type,
                build_config,
                dead_code_graph,
                dependency_graph),
            TypedMatchCondition::CatchAll,
            warnings,
            errors
        );
        */

        unimplemented!()
    }

    /// Makes a fresh copy of all type ids in this expression. Used when monomorphizing.
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        unimplemented!()
    }
}
