use std::collections::{HashMap, HashSet};

use crate::control_flow_analysis::ControlFlowGraph;
use crate::error::ok;
use crate::type_engine::TypeId;
use crate::{BuildConfig, CompileResult, MatchCondition, Namespace, Span, TypeInfo, TypeParameter};

use super::{TypedScrutinee, TypedScrutineeVariant};

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) enum TypedMatchCondition<'sc> {
    CatchAll(TypedCatchAll<'sc>),
    Scrutinee(TypedScrutinee<'sc>),
}

#[derive(Clone, Debug)]
pub(crate) struct TypedCatchAll<'sc> {
    pub span: Span<'sc>,
}

impl<'sc> TypedMatchCondition<'sc> {
    pub(crate) fn type_check(
        other: MatchCondition<'sc>,
        namespace: &mut Namespace<'sc>,
        primary_expression_type: TypeId,
        help_text: impl Into<String> + Clone,
        self_type: TypeId,
        build_config: &BuildConfig,
        dead_code_graph: &mut ControlFlowGraph<'sc>,
        dependency_graph: &mut HashMap<String, HashSet<String>>,
    ) -> CompileResult<'sc, Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let typed_condition = match other {
            MatchCondition::CatchAll(catch_all) => TypedMatchCondition::CatchAll(TypedCatchAll {
                span: catch_all.span.clone(),
            }),
            MatchCondition::Scrutinee(scrutinee) => {
                let typed_scrutinee = check!(
                    TypedScrutinee::type_check(
                        scrutinee.clone(),
                        namespace,
                        primary_expression_type,
                        help_text,
                        self_type,
                        build_config,
                        dead_code_graph,
                        dependency_graph
                    ),
                    TypedScrutinee {
                        scrutinee: TypedScrutineeVariant::Unit {
                            span: scrutinee.span()
                        },
                        return_type: crate::type_engine::insert_type(TypeInfo::ErrorRecovery),
                        span: scrutinee.span()
                    },
                    warnings,
                    errors
                );
                TypedMatchCondition::Scrutinee(typed_scrutinee)
            }
        };
        ok(typed_condition, warnings, errors)
    }

    /// Makes a fresh copy of all type ids in this expression. Used when monomorphizing.
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        unimplemented!()
    }
}
