use crate::{
    decl_engine::*,
    engine_threading::*,
    has_changes,
    language::ty::*,
    semantic_analysis::{
        TypeCheckAnalysis, TypeCheckAnalysisContext, TypeCheckContext, TypeCheckFinalization,
        TypeCheckFinalizationContext,
    },
    type_system::*,
};
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    hash::{Hash, Hasher},
};
use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::{Ident, Span, Spanned};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TyReassignment {
    pub lhs: TyReassignmentTarget,
    pub rhs: TyExpression,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TyReassignmentTarget {
    /// An [TyExpression] representing a single variable or a path
    /// to a part of an aggregate.
    /// E.g.:
    ///  - `my_variable`
    ///  - `array[0].field.x.1`
    ElementAccess {
        /// [Ident] of the single variable, or the starting variable
        /// of the path to a part of an aggregate.
        base_name: Ident,
        /// [TypeId] of the variable behind the `base_name`.
        base_type: TypeId,
        /// Indices representing the path from the `base_name` to the
        /// final part of an aggregate.
        /// Empty if the LHS of the reassignment is a single variable.
        indices: Vec<ProjectionKind>,
    },
    /// An dereferencing [TyExpression] representing dereferencing
    /// of an arbitrary reference expression with optional indices.
    /// E.g.:
    ///  - *my_ref
    ///  - **if x > 0 { &mut &mut a } else { &mut &mut b }
    ///  - (*my_ref)\[0\]
    ///  - (**my_ref)\[0\]
    ///
    /// The [TyExpression] is guaranteed to be of [TyExpressionVariant::Deref].
    DerefAccess {
        /// [TyExpression] of one or multiple nested [TyExpressionVariant::Deref].
        exp: Box<TyExpression>,
        /// Indices representing the path from the `base_name` to the
        /// final part of an aggregate.
        /// Empty if the LHS of the reassignment is a single variable.
        indices: Vec<ProjectionKind>,
    },
}

impl EqWithEngines for TyReassignmentTarget {}
impl PartialEqWithEngines for TyReassignmentTarget {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        let type_engine = ctx.engines().te();
        match (self, other) {
            (
                TyReassignmentTarget::DerefAccess {
                    exp: l,
                    indices: l_indices,
                },
                TyReassignmentTarget::DerefAccess {
                    exp: r,
                    indices: r_indices,
                },
            ) => (*l).eq(r, ctx) && l_indices.eq(r_indices, ctx),
            (
                TyReassignmentTarget::ElementAccess {
                    base_name: l_name,
                    base_type: l_type,
                    indices: l_indices,
                },
                TyReassignmentTarget::ElementAccess {
                    base_name: r_name,
                    base_type: r_type,
                    indices: r_indices,
                },
            ) => {
                l_name == r_name
                    && (l_type == r_type
                        || type_engine.get(*l_type).eq(&type_engine.get(*r_type), ctx))
                    && l_indices.eq(r_indices, ctx)
            }
            _ => false,
        }
    }
}

impl EqWithEngines for TyReassignment {}
impl PartialEqWithEngines for TyReassignment {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.lhs.eq(&other.lhs, ctx) && self.rhs.eq(&other.rhs, ctx)
    }
}

impl HashWithEngines for TyReassignmentTarget {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        let type_engine = engines.te();
        match self {
            TyReassignmentTarget::DerefAccess { exp, indices } => {
                exp.hash(state, engines);
                indices.hash(state, engines);
            }
            TyReassignmentTarget::ElementAccess {
                base_name,
                base_type,
                indices,
            } => {
                base_name.hash(state);
                type_engine.get(*base_type).hash(state, engines);
                indices.hash(state, engines);
            }
        };
    }
}

impl HashWithEngines for TyReassignment {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        let TyReassignment { lhs, rhs } = self;

        lhs.hash(state, engines);
        rhs.hash(state, engines);
    }
}

impl SubstTypes for TyReassignmentTarget {
    fn subst_inner(&mut self, ctx: &SubstTypesContext) -> HasChanges {
        has_changes! {
            match self {
                TyReassignmentTarget::DerefAccess{exp, indices} => {
                    has_changes! {
                        exp.subst(ctx);
                        indices.subst(ctx);
                    }
                },
                TyReassignmentTarget::ElementAccess { base_type, indices, .. } => {
                    has_changes! {
                        base_type.subst(ctx);
                        indices.subst(ctx);
                    }
                }
            };
        }
    }
}

impl SubstTypes for TyReassignment {
    fn subst_inner(&mut self, ctx: &SubstTypesContext) -> HasChanges {
        has_changes! {
            self.lhs.subst(ctx);
            self.rhs.subst(ctx);
        }
    }
}

impl ReplaceDecls for TyReassignmentTarget {
    fn replace_decls_inner(
        &mut self,
        decl_mapping: &DeclMapping,
        handler: &Handler,
        ctx: &mut TypeCheckContext,
    ) -> Result<bool, ErrorEmitted> {
        Ok(match self {
            TyReassignmentTarget::DerefAccess { exp, indices } => {
                let mut changed = exp.replace_decls(decl_mapping, handler, ctx)?;
                changed |= indices
                    .iter_mut()
                    .map(|i| i.replace_decls(decl_mapping, handler, ctx))
                    .collect::<Result<Vec<bool>, _>>()?
                    .iter()
                    .any(|is_changed| *is_changed);
                changed
            }
            TyReassignmentTarget::ElementAccess { indices, .. } => indices
                .iter_mut()
                .map(|i| i.replace_decls(decl_mapping, handler, ctx))
                .collect::<Result<Vec<bool>, _>>()?
                .iter()
                .any(|is_changed| *is_changed),
        })
    }
}

impl ReplaceDecls for TyReassignment {
    fn replace_decls_inner(
        &mut self,
        decl_mapping: &DeclMapping,
        handler: &Handler,
        ctx: &mut TypeCheckContext,
    ) -> Result<bool, ErrorEmitted> {
        let lhs_changed = self.lhs.replace_decls(decl_mapping, handler, ctx)?;
        let rhs_changed = self.rhs.replace_decls(decl_mapping, handler, ctx)?;

        Ok(lhs_changed || rhs_changed)
    }
}

impl TypeCheckAnalysis for TyReassignmentTarget {
    fn type_check_analyze(
        &self,
        handler: &Handler,
        ctx: &mut TypeCheckAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        match self {
            TyReassignmentTarget::DerefAccess { exp, indices } => {
                exp.type_check_analyze(handler, ctx)?;
                indices
                    .iter()
                    .map(|i| i.type_check_analyze(handler, ctx))
                    .collect::<Result<Vec<()>, _>>()
                    .map(|_| ())?
            }
            TyReassignmentTarget::ElementAccess { indices, .. } => indices
                .iter()
                .map(|i| i.type_check_analyze(handler, ctx))
                .collect::<Result<Vec<()>, _>>()
                .map(|_| ())?,
        };
        Ok(())
    }
}

impl TypeCheckAnalysis for TyReassignment {
    fn type_check_analyze(
        &self,
        handler: &Handler,
        ctx: &mut TypeCheckAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        self.lhs.type_check_analyze(handler, ctx)?;
        self.rhs.type_check_analyze(handler, ctx)?;

        Ok(())
    }
}

impl TypeCheckFinalization for TyReassignmentTarget {
    fn type_check_finalize(
        &mut self,
        handler: &Handler,
        ctx: &mut TypeCheckFinalizationContext,
    ) -> Result<(), ErrorEmitted> {
        match self {
            TyReassignmentTarget::DerefAccess { exp, indices } => {
                exp.type_check_finalize(handler, ctx)?;
                indices
                    .iter_mut()
                    .map(|i| i.type_check_finalize(handler, ctx))
                    .collect::<Result<Vec<()>, _>>()
                    .map(|_| ())?;
            }
            TyReassignmentTarget::ElementAccess { indices, .. } => indices
                .iter_mut()
                .map(|i| i.type_check_finalize(handler, ctx))
                .collect::<Result<Vec<()>, _>>()
                .map(|_| ())?,
        };
        Ok(())
    }
}

impl TypeCheckFinalization for TyReassignment {
    fn type_check_finalize(
        &mut self,
        handler: &Handler,
        ctx: &mut TypeCheckFinalizationContext,
    ) -> Result<(), ErrorEmitted> {
        self.lhs.type_check_finalize(handler, ctx)?;
        self.rhs.type_check_finalize(handler, ctx)?;

        Ok(())
    }
}

impl UpdateConstantExpression for TyReassignmentTarget {
    fn update_constant_expression(&mut self, engines: &Engines, implementing_type: &TyDecl) {
        match self {
            TyReassignmentTarget::DerefAccess { exp, indices } => {
                exp.update_constant_expression(engines, implementing_type);
                indices
                    .iter_mut()
                    .for_each(|i| i.update_constant_expression(engines, implementing_type));
            }
            TyReassignmentTarget::ElementAccess { indices, .. } => {
                indices
                    .iter_mut()
                    .for_each(|i| i.update_constant_expression(engines, implementing_type));
            }
        };
    }
}

impl UpdateConstantExpression for TyReassignment {
    fn update_constant_expression(&mut self, engines: &Engines, implementing_type: &TyDecl) {
        self.lhs
            .update_constant_expression(engines, implementing_type);
        self.rhs
            .update_constant_expression(engines, implementing_type);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProjectionKind {
    StructField {
        name: Ident,
        field_to_access: Option<Box<TyStructField>>,
    },
    TupleField {
        index: usize,
        index_span: Span,
    },
    ArrayIndex {
        index: Box<TyExpression>,
        index_span: Span,
    },
}

impl EqWithEngines for ProjectionKind {}
impl PartialEqWithEngines for ProjectionKind {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        match (self, other) {
            (
                ProjectionKind::StructField {
                    name: l_name,
                    field_to_access: _,
                },
                ProjectionKind::StructField {
                    name: r_name,
                    field_to_access: _,
                },
            ) => l_name == r_name,
            (
                ProjectionKind::TupleField {
                    index: l_index,
                    index_span: l_index_span,
                },
                ProjectionKind::TupleField {
                    index: r_index,
                    index_span: r_index_span,
                },
            ) => l_index == r_index && l_index_span == r_index_span,
            (
                ProjectionKind::ArrayIndex {
                    index: l_index,
                    index_span: l_index_span,
                },
                ProjectionKind::ArrayIndex {
                    index: r_index,
                    index_span: r_index_span,
                },
            ) => l_index.eq(r_index, ctx) && l_index_span == r_index_span,
            _ => false,
        }
    }
}

impl HashWithEngines for ProjectionKind {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        use ProjectionKind::*;
        std::mem::discriminant(self).hash(state);
        match self {
            StructField {
                name,
                field_to_access: _,
            } => name.hash(state),
            TupleField {
                index,
                // these fields are not hashed because they aren't relevant/a
                // reliable source of obj v. obj distinction
                index_span: _,
            } => index.hash(state),
            ArrayIndex {
                index,
                // these fields are not hashed because they aren't relevant/a
                // reliable source of obj v. obj distinction
                index_span: _,
            } => {
                index.hash(state, engines);
            }
        }
    }
}

impl SubstTypes for ProjectionKind {
    fn subst_inner(&mut self, ctx: &SubstTypesContext) -> HasChanges {
        use ProjectionKind::*;
        match self {
            ArrayIndex { index, .. } => index.subst(ctx),
            _ => HasChanges::No,
        }
    }
}

impl ReplaceDecls for ProjectionKind {
    fn replace_decls_inner(
        &mut self,
        decl_mapping: &DeclMapping,
        handler: &Handler,
        ctx: &mut TypeCheckContext,
    ) -> Result<bool, ErrorEmitted> {
        use ProjectionKind::*;
        match self {
            ArrayIndex { index, .. } => index.replace_decls(decl_mapping, handler, ctx),
            _ => Ok(false),
        }
    }
}

impl TypeCheckAnalysis for ProjectionKind {
    fn type_check_analyze(
        &self,
        handler: &Handler,
        ctx: &mut TypeCheckAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        use ProjectionKind::*;
        match self {
            ArrayIndex { index, .. } => index.type_check_analyze(handler, ctx),
            _ => Ok(()),
        }
    }
}

impl TypeCheckFinalization for ProjectionKind {
    fn type_check_finalize(
        &mut self,
        handler: &Handler,
        ctx: &mut TypeCheckFinalizationContext,
    ) -> Result<(), ErrorEmitted> {
        use ProjectionKind::*;
        match self {
            ArrayIndex { index, .. } => index.type_check_finalize(handler, ctx),
            _ => Ok(()),
        }
    }
}

impl UpdateConstantExpression for ProjectionKind {
    fn update_constant_expression(&mut self, engines: &Engines, implementing_type: &TyDecl) {
        use ProjectionKind::*;
        #[allow(clippy::single_match)]
        // To keep it consistent and same looking as the above implementations.
        match self {
            ArrayIndex { index, .. } => {
                index.update_constant_expression(engines, implementing_type)
            }
            _ => (),
        }
    }
}

impl Spanned for ProjectionKind {
    fn span(&self) -> Span {
        match self {
            ProjectionKind::StructField {
                name,
                field_to_access: _,
            } => name.span(),
            ProjectionKind::TupleField { index_span, .. } => index_span.clone(),
            ProjectionKind::ArrayIndex { index_span, .. } => index_span.clone(),
        }
    }
}

impl ProjectionKind {
    pub(crate) fn pretty_print(&self) -> Cow<'_, str> {
        match self {
            ProjectionKind::StructField {
                name,
                field_to_access: _,
            } => Cow::Borrowed(name.as_str()),
            ProjectionKind::TupleField { index, .. } => Cow::Owned(index.to_string()),
            ProjectionKind::ArrayIndex { index, .. } => Cow::Owned(format!("{index:#?}")),
        }
    }
}
