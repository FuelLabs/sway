use std::collections::{HashMap, HashSet};

use super::*;
use crate::control_flow_analysis::ControlFlowGraph;
use crate::error::ok;
use crate::type_engine::TypeId;
use crate::{BuildConfig, CompileResult, MatchBranch, Namespace, Span, TypeParameter};

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) struct TypedMatchBranch<'sc> {
    pub condition: TypedMatchCondition<'sc>,
    pub result: TypedExpression<'sc>,
    pub span: Span<'sc>,
}

impl<'sc> TypedMatchBranch<'sc> {
    pub(crate) fn type_check(
        other: MatchBranch<'sc>,
        primary_expression_type: TypeId,
        namespace: &mut Namespace<'sc>,
        type_annotation: Option<TypeId>,
        help_text: impl Into<String> + Clone,
        self_type: TypeId,
        build_config: &BuildConfig,
        dead_code_graph: &mut ControlFlowGraph<'sc>,
        dependency_graph: &mut HashMap<String, HashSet<String>>,
    ) -> CompileResult<'sc, Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let condition = other.condition;
        let result = other.result;
        let span = other.span;
        let typed_condition = check!(
            TypedMatchCondition::type_check(
                condition,
                namespace,
                primary_expression_type,
                "all branch conditions must be the same type",
                self_type,
                build_config,
                dead_code_graph,
                dependency_graph
            ),
            TypedMatchCondition::CatchAll(TypedCatchAll { span: span.clone() }),
            warnings,
            errors
        );
        let typed_result = check!(
            TypedExpression::type_check(
                result,
                namespace,
                type_annotation,
                "all branches must be the same type",
                self_type,
                build_config,
                dead_code_graph,
                dependency_graph
            ),
            error_recovery_expr(span.clone()),
            warnings,
            errors
        );
        let branch = TypedMatchBranch {
            condition: typed_condition,
            result: typed_result,
            span,
        };
        ok(branch, warnings, errors)
    }

    /// Makes a fresh copy of all type ids in this expression. Used when monomorphizing.
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        unimplemented!()
    }
}
