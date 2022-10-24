use std::fmt;

use derivative::Derivative;
use sway_types::Span;

use crate::{
    declaration_engine::de_get_function,
    error::*,
    language::{parsed, ty::*},
    type_system::*,
    types::DeterministicallyAborts,
};

#[derive(Clone, Debug, Eq, Derivative)]
#[derivative(PartialEq)]
pub struct TyAstNode {
    pub content:     TyAstNodeContent,
    #[derivative(PartialEq = "ignore")]
    pub(crate) span: Span,
}

impl fmt::Display for TyAstNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use TyAstNodeContent::*;
        let text = match &self.content {
            Declaration(ref typed_decl) => typed_decl.to_string(),
            Expression(exp) => exp.to_string(),
            ImplicitReturnExpression(exp) => format!("return {}", exp),
            SideEffect => "".into(),
        };
        f.write_str(&text)
    }
}

impl CopyTypes for TyAstNode {
    fn copy_types_inner(&mut self, type_mapping: &TypeMapping) {
        match self.content {
            TyAstNodeContent::ImplicitReturnExpression(ref mut exp) => exp.copy_types(type_mapping),
            TyAstNodeContent::Declaration(ref mut decl) => decl.copy_types(type_mapping),
            TyAstNodeContent::Expression(ref mut expr) => expr.copy_types(type_mapping),
            TyAstNodeContent::SideEffect => (),
        }
    }
}

impl CollectTypesMetadata for TyAstNode {
    fn collect_types_metadata(
        &self,
        ctx: &mut CollectTypesMetadataContext,
    ) -> CompileResult<Vec<TypeMetadata>> {
        self.content.collect_types_metadata(ctx)
    }
}

impl DeterministicallyAborts for TyAstNode {
    fn deterministically_aborts(&self) -> bool {
        use TyAstNodeContent::*;
        match &self.content {
            Declaration(_) => false,
            Expression(exp) | ImplicitReturnExpression(exp) => exp.deterministically_aborts(),
            SideEffect => false,
        }
    }
}

impl TyAstNode {
    /// recurse into `self` and get any return statements -- used to validate that all returns
    /// do indeed return the correct type
    /// This does _not_ extract implicit return statements as those are not control flow! This is
    /// _only_ for explicit returns.
    pub(crate) fn gather_return_statements(&self) -> Vec<&TyExpression> {
        match &self.content {
            TyAstNodeContent::ImplicitReturnExpression(ref exp) => exp.gather_return_statements(),
            // assignments and  reassignments can happen during control flow and can abort
            TyAstNodeContent::Declaration(TyDeclaration::VariableDeclaration(decl)) => {
                decl.body.gather_return_statements()
            }
            TyAstNodeContent::Expression(exp) => exp.gather_return_statements(),
            TyAstNodeContent::SideEffect | TyAstNodeContent::Declaration(_) => vec![],
        }
    }

    /// Returns `true` if this AST node will be exported in a library, i.e. it is a public declaration.
    pub(crate) fn is_public(&self) -> CompileResult<bool> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let public = match &self.content {
            TyAstNodeContent::Declaration(decl) => {
                let visibility = check!(
                    decl.visibility(),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                visibility.is_public()
            }
            TyAstNodeContent::Expression(_)
            | TyAstNodeContent::SideEffect
            | TyAstNodeContent::ImplicitReturnExpression(_) => false,
        };
        ok(public, warnings, errors)
    }

    /// Naive check to see if this node is a function declaration of a function called `main` if
    /// the [TreeType] is Script or Predicate.
    pub(crate) fn is_main_function(&self, tree_type: parsed::TreeType) -> CompileResult<bool> {
        let mut warnings = vec![];
        let mut errors = vec![];
        match &self {
            TyAstNode {
                span,
                content: TyAstNodeContent::Declaration(TyDeclaration::FunctionDeclaration(decl_id)),
                ..
            } => {
                let TyFunctionDeclaration { name, .. } = check!(
                    CompileResult::from(de_get_function(decl_id.clone(), span)),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let is_main = name.as_str() == sway_types::constants::DEFAULT_ENTRY_POINT_FN_NAME
                    && matches!(
                        tree_type,
                        parsed::TreeType::Script | parsed::TreeType::Predicate
                    );
                ok(is_main, warnings, errors)
            }
            _ => ok(false, warnings, errors),
        }
    }

    pub(crate) fn type_info(&self) -> TypeInfo {
        // return statement should be ()
        match &self.content {
            TyAstNodeContent::Declaration(_) => TypeInfo::Tuple(Vec::new()),
            TyAstNodeContent::Expression(TyExpression { return_type, .. }) => {
                look_up_type_id(*return_type)
            }
            TyAstNodeContent::ImplicitReturnExpression(TyExpression { return_type, .. }) => {
                look_up_type_id(*return_type)
            }
            TyAstNodeContent::SideEffect => TypeInfo::Tuple(Vec::new()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TyAstNodeContent {
    Declaration(TyDeclaration),
    Expression(TyExpression),
    ImplicitReturnExpression(TyExpression),
    // a no-op node used for something that just issues a side effect, like an import statement.
    SideEffect,
}

impl CollectTypesMetadata for TyAstNodeContent {
    fn collect_types_metadata(
        &self,
        ctx: &mut CollectTypesMetadataContext,
    ) -> CompileResult<Vec<TypeMetadata>> {
        use TyAstNodeContent::*;
        match self {
            Declaration(decl) => decl.collect_types_metadata(ctx),
            Expression(expr) => expr.collect_types_metadata(ctx),
            ImplicitReturnExpression(expr) => expr.collect_types_metadata(ctx),
            SideEffect => ok(vec![], vec![], vec![]),
        }
    }
}
