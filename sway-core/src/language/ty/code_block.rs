use crate::{
    decl_engine::*, engine_threading::*, language::ty::*, semantic_analysis::TypeCheckContext,
    transform::AllowDeprecatedState,
};
use serde::{Deserialize, Serialize};
use std::hash::Hasher;
use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::Span;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TyCodeBlock {
    pub contents: Vec<TyAstNode>,
    pub(crate) whole_block_span: Span,
}

impl TyCodeBlock {
    pub(crate) fn check_deprecated(
        &self,
        engines: &Engines,
        handler: &Handler,
        allow_deprecated: &mut AllowDeprecatedState,
    ) {
        for n in self.contents.iter() {
            n.check_deprecated(engines, handler, allow_deprecated);
        }
    }
}

impl Default for TyCodeBlock {
    fn default() -> Self {
        Self {
            contents: Default::default(),
            whole_block_span: Span::dummy(),
        }
    }
}

impl EqWithEngines for TyCodeBlock {}
impl PartialEqWithEngines for TyCodeBlock {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.contents.eq(&other.contents, ctx)
    }
}

impl HashWithEngines for TyCodeBlock {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        let TyCodeBlock { contents, .. } = self;
        contents.hash(state, engines);
    }
}

// impl SubstTypes for TyCodeBlock {
//     fn subst_inner(&mut self, ctx: &SubstTypesContext) -> HasChanges {
//         self.contents.subst(ctx)
//     }
// }

impl ReplaceDecls for TyCodeBlock {
    fn replace_decls_inner(
        &mut self,
        decl_mapping: &DeclMapping,
        handler: &Handler,
        ctx: &mut TypeCheckContext,
    ) -> Result<bool, ErrorEmitted> {
        handler.scope(|handler| {
            let mut has_changes = false;
            for node in self.contents.iter_mut() {
                if let Ok(r) = node.replace_decls(decl_mapping, handler, ctx) {
                    has_changes |= r;
                }
            }
            Ok(has_changes)
        })
    }
}

impl UpdateConstantExpression for TyCodeBlock {
    fn update_constant_expression(&mut self, engines: &Engines, implementing_type: &TyDecl) {
        self.contents
            .iter_mut()
            .for_each(|x| x.update_constant_expression(engines, implementing_type));
    }
}

impl MaterializeConstGenerics for TyCodeBlock {
    fn materialize_const_generics(
        &mut self,
        engines: &Engines,
        handler: &Handler,
        name: &str,
        value: &TyExpression,
    ) -> Result<(), ErrorEmitted> {
        self.contents
            .iter_mut()
            .try_for_each(|x| x.materialize_const_generics(engines, handler, name, value))
    }
}
