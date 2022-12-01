use std::fmt::{self, Debug};

use sway_types::{Ident, Span};

use crate::{
    declaration_engine::{de_get_function, DeclMapping, ReplaceDecls},
    error::*,
    language::{parsed, ty::*},
    transform::AttributeKind,
    type_system::*,
    types::DeterministicallyAborts,
};

pub trait GetDeclIdent {
    fn get_decl_ident(&self) -> Option<Ident>;
}

#[derive(Clone, Debug)]
pub struct TyAstNode {
    pub content: TyAstNodeContent,
    pub(crate) span: Span,
}

impl EqWithTypeEngine for TyAstNode {}
impl PartialEqWithTypeEngine for TyAstNode {
    fn eq(&self, rhs: &Self, type_engine: &TypeEngine) -> bool {
        self.content.eq(&rhs.content, type_engine)
    }
}

impl DisplayWithTypeEngine for TyAstNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, type_engine: &TypeEngine) -> fmt::Result {
        use TyAstNodeContent::*;
        match &self.content {
            Declaration(typed_decl) => DisplayWithTypeEngine::fmt(typed_decl, f, type_engine),
            Expression(exp) => DisplayWithTypeEngine::fmt(exp, f, type_engine),
            ImplicitReturnExpression(exp) => write!(f, "return {}", type_engine.help_out(exp)),
            SideEffect => f.write_str(""),
        }
    }
}

impl CopyTypes for TyAstNode {
    fn copy_types_inner(&mut self, type_mapping: &TypeMapping, type_engine: &TypeEngine) {
        match self.content {
            TyAstNodeContent::ImplicitReturnExpression(ref mut exp) => {
                exp.copy_types(type_mapping, type_engine)
            }
            TyAstNodeContent::Declaration(ref mut decl) => {
                decl.copy_types(type_mapping, type_engine)
            }
            TyAstNodeContent::Expression(ref mut expr) => {
                expr.copy_types(type_mapping, type_engine)
            }
            TyAstNodeContent::SideEffect => (),
        }
    }
}

impl ReplaceSelfType for TyAstNode {
    fn replace_self_type(&mut self, type_engine: &TypeEngine, self_type: TypeId) {
        match self.content {
            TyAstNodeContent::ImplicitReturnExpression(ref mut exp) => {
                exp.replace_self_type(type_engine, self_type)
            }
            TyAstNodeContent::Declaration(ref mut decl) => {
                decl.replace_self_type(type_engine, self_type)
            }
            TyAstNodeContent::Expression(ref mut expr) => {
                expr.replace_self_type(type_engine, self_type)
            }
            TyAstNodeContent::SideEffect => (),
        }
    }
}

impl ReplaceDecls for TyAstNode {
    fn replace_decls_inner(&mut self, decl_mapping: &DeclMapping, type_engine: &TypeEngine) {
        match self.content {
            TyAstNodeContent::ImplicitReturnExpression(ref mut exp) => {
                exp.replace_decls(decl_mapping, type_engine)
            }
            TyAstNodeContent::Declaration(_) => {}
            TyAstNodeContent::Expression(ref mut expr) => {
                expr.replace_decls(decl_mapping, type_engine)
            }
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
    fn deterministically_aborts(&self, check_call_body: bool) -> bool {
        use TyAstNodeContent::*;
        match &self.content {
            Declaration(_) => false,
            Expression(exp) | ImplicitReturnExpression(exp) => {
                exp.deterministically_aborts(check_call_body)
            }
            SideEffect => false,
        }
    }
}

impl GetDeclIdent for TyAstNode {
    fn get_decl_ident(&self) -> Option<Ident> {
        self.content.get_decl_ident()
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

    /// Check to see if this node is a function declaration of a function annotated as test.
    pub(crate) fn is_test_function(&self) -> CompileResult<bool> {
        let mut warnings = vec![];
        let mut errors = vec![];
        match &self {
            TyAstNode {
                span,
                content: TyAstNodeContent::Declaration(TyDeclaration::FunctionDeclaration(decl_id)),
                ..
            } => {
                let TyFunctionDeclaration { attributes, .. } = check!(
                    CompileResult::from(de_get_function(decl_id.clone(), span)),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                ok(
                    attributes.contains_key(&AttributeKind::Test),
                    warnings,
                    errors,
                )
            }
            _ => ok(false, warnings, errors),
        }
    }

    pub(crate) fn type_info(&self, type_engine: &TypeEngine) -> TypeInfo {
        // return statement should be ()
        match &self.content {
            TyAstNodeContent::Declaration(_) => TypeInfo::Tuple(Vec::new()),
            TyAstNodeContent::Expression(TyExpression { return_type, .. }) => {
                type_engine.look_up_type_id(*return_type)
            }
            TyAstNodeContent::ImplicitReturnExpression(TyExpression { return_type, .. }) => {
                type_engine.look_up_type_id(*return_type)
            }
            TyAstNodeContent::SideEffect => TypeInfo::Tuple(Vec::new()),
        }
    }
}

#[derive(Clone, Debug)]
pub enum TyAstNodeContent {
    Declaration(TyDeclaration),
    Expression(TyExpression),
    ImplicitReturnExpression(TyExpression),
    // a no-op node used for something that just issues a side effect, like an import statement.
    SideEffect,
}

impl EqWithTypeEngine for TyAstNodeContent {}
impl PartialEqWithTypeEngine for TyAstNodeContent {
    fn eq(&self, rhs: &Self, type_engine: &TypeEngine) -> bool {
        match (self, rhs) {
            (Self::Declaration(x), Self::Declaration(y)) => x.eq(y, type_engine),
            (Self::Expression(x), Self::Expression(y)) => x.eq(y, type_engine),
            (Self::ImplicitReturnExpression(x), Self::ImplicitReturnExpression(y)) => {
                x.eq(y, type_engine)
            }
            (Self::SideEffect, Self::SideEffect) => true,
            _ => false,
        }
    }
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

impl GetDeclIdent for TyAstNodeContent {
    fn get_decl_ident(&self) -> Option<Ident> {
        match self {
            TyAstNodeContent::Declaration(decl) => decl.get_decl_ident(),
            TyAstNodeContent::Expression(_expr) => None, //expr.get_decl_ident(),
            TyAstNodeContent::ImplicitReturnExpression(_expr) => None, //expr.get_decl_ident(),
            TyAstNodeContent::SideEffect => None,
        }
    }
}
