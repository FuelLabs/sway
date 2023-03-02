use std::{
    fmt::{self, Debug},
    hash::{Hash, Hasher},
};

use sway_error::error::CompileError;
use sway_types::{Ident, Span};

use crate::{
    decl_engine::*,
    engine_threading::*,
    error::*,
    language::{parsed::TreeType, ty::*, Visibility},
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

impl EqWithEngines for TyAstNode {}
impl PartialEqWithEngines for TyAstNode {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        self.content.eq(&other.content, engines)
    }
}

impl HashWithEngines for TyAstNode {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        let TyAstNode {
            content,
            // the span is not hashed because it isn't relevant/a reliable
            // source of obj v. obj distinction
            span: _,
        } = self;
        content.hash(state, engines);
    }
}

impl DisplayWithEngines for TyAstNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: Engines<'_>) -> fmt::Result {
        use TyAstNodeContent::*;
        match &self.content {
            Declaration(typed_decl) => DisplayWithEngines::fmt(typed_decl, f, engines),
            Expression(exp) => DisplayWithEngines::fmt(exp, f, engines),
            ImplicitReturnExpression(exp) => write!(f, "return {}", engines.help_out(exp)),
            SideEffect(_) => f.write_str(""),
        }
    }
}

impl SubstTypes for TyAstNode {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        match self.content {
            TyAstNodeContent::ImplicitReturnExpression(ref mut exp) => {
                exp.subst(type_mapping, engines)
            }
            TyAstNodeContent::Declaration(ref mut decl) => decl.subst(type_mapping, engines),
            TyAstNodeContent::Expression(ref mut expr) => expr.subst(type_mapping, engines),
            TyAstNodeContent::SideEffect(_) => (),
        }
    }
}

impl ReplaceSelfType for TyAstNode {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        match self.content {
            TyAstNodeContent::ImplicitReturnExpression(ref mut exp) => {
                exp.replace_self_type(engines, self_type)
            }
            TyAstNodeContent::Declaration(ref mut decl) => {
                decl.replace_self_type(engines, self_type)
            }
            TyAstNodeContent::Expression(ref mut expr) => {
                expr.replace_self_type(engines, self_type)
            }
            TyAstNodeContent::SideEffect(_) => (),
        }
    }
}

impl ReplaceDecls for TyAstNode {
    fn replace_decls_inner(&mut self, decl_mapping: &DeclMapping, engines: Engines<'_>) {
        match self.content {
            TyAstNodeContent::ImplicitReturnExpression(ref mut exp) => {
                exp.replace_decls(decl_mapping, engines)
            }
            TyAstNodeContent::Declaration(_) => {}
            TyAstNodeContent::Expression(ref mut expr) => expr.replace_decls(decl_mapping, engines),
            TyAstNodeContent::SideEffect(_) => (),
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
    fn deterministically_aborts(&self, decl_engine: &DeclEngine, check_call_body: bool) -> bool {
        use TyAstNodeContent::*;
        match &self.content {
            Declaration(_) => false,
            Expression(exp) | ImplicitReturnExpression(exp) => {
                exp.deterministically_aborts(decl_engine, check_call_body)
            }
            SideEffect(_) => false,
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
            TyAstNodeContent::SideEffect(_) | TyAstNodeContent::Declaration(_) => vec![],
        }
    }

    /// Returns `true` if this AST node will be exported in a library, i.e. it is a public declaration.
    pub(crate) fn is_public(&self, decl_engine: &DeclEngine) -> CompileResult<bool> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let public = match &self.content {
            TyAstNodeContent::Declaration(decl) => {
                let visibility = check!(
                    decl.visibility(decl_engine),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                visibility.is_public()
            }
            TyAstNodeContent::Expression(_)
            | TyAstNodeContent::SideEffect(_)
            | TyAstNodeContent::ImplicitReturnExpression(_) => false,
        };
        ok(public, warnings, errors)
    }

    /// Check to see if this node is a function declaration with generic type parameters.
    pub(crate) fn is_generic_function(&self, decl_engine: &DeclEngine) -> bool {
        match &self {
            TyAstNode {
                span: _,
                content:
                    TyAstNodeContent::Declaration(TyDeclaration::FunctionDeclaration {
                        decl_id, ..
                    }),
                ..
            } => {
                let TyFunctionDeclaration {
                    type_parameters, ..
                } = decl_engine.get_function(decl_id);
                !type_parameters.is_empty()
            }
            _ => false,
        }
    }

    /// Check to see if this node is a function declaration of a function annotated as test.
    pub(crate) fn is_test_function(&self, decl_engine: &DeclEngine) -> bool {
        match &self {
            TyAstNode {
                span: _,
                content:
                    TyAstNodeContent::Declaration(TyDeclaration::FunctionDeclaration {
                        decl_id, ..
                    }),
                ..
            } => {
                let TyFunctionDeclaration { attributes, .. } = decl_engine.get_function(decl_id);
                attributes.contains_key(&AttributeKind::Test)
            }
            _ => false,
        }
    }

    pub(crate) fn is_entry_point(
        &self,
        decl_engine: &DeclEngine,
        tree_type: &TreeType,
    ) -> Result<bool, CompileError> {
        match tree_type {
            TreeType::Predicate | TreeType::Script => {
                // Predicates and scripts have main and test functions as entry points.
                match self {
                    TyAstNode {
                        span: _,
                        content:
                            TyAstNodeContent::Declaration(TyDeclaration::FunctionDeclaration {
                                decl_id,
                                ..
                            }),
                        ..
                    } => {
                        let decl = decl_engine.get_function(decl_id);
                        Ok(decl.is_entry())
                    }
                    _ => Ok(false),
                }
            }
            TreeType::Contract | TreeType::Library { .. } => match self {
                TyAstNode {
                    content:
                        TyAstNodeContent::Declaration(TyDeclaration::FunctionDeclaration {
                            decl_id,
                            decl_span: _,
                            ..
                        }),
                    ..
                } => {
                    let decl = decl_engine.get_function(decl_id);
                    Ok(decl.visibility == Visibility::Public || decl.is_test())
                }
                TyAstNode {
                    content:
                        TyAstNodeContent::Declaration(TyDeclaration::TraitDeclaration {
                            decl_id,
                            decl_span: _,
                            ..
                        }),
                    ..
                } => Ok(decl_engine.get_trait(decl_id).visibility.is_public()),
                TyAstNode {
                    content:
                        TyAstNodeContent::Declaration(TyDeclaration::StructDeclaration {
                            decl_id,
                            decl_span: _,
                            ..
                        }),
                    ..
                } => {
                    let struct_decl = decl_engine.get_struct(decl_id);
                    Ok(struct_decl.visibility == Visibility::Public)
                }
                TyAstNode {
                    content: TyAstNodeContent::Declaration(TyDeclaration::ImplTrait { .. }),
                    ..
                } => Ok(true),
                TyAstNode {
                    content:
                        TyAstNodeContent::Declaration(TyDeclaration::ConstantDeclaration {
                            decl_id,
                            decl_span: _,
                            ..
                        }),
                    ..
                } => {
                    let decl = decl_engine.get_constant(decl_id);
                    Ok(decl.visibility.is_public())
                }
                _ => Ok(false),
            },
        }
    }

    pub(crate) fn type_info(&self, type_engine: &TypeEngine) -> TypeInfo {
        // return statement should be ()
        match &self.content {
            TyAstNodeContent::Declaration(_) => TypeInfo::Tuple(Vec::new()),
            TyAstNodeContent::Expression(TyExpression { return_type, .. }) => {
                type_engine.get(*return_type)
            }
            TyAstNodeContent::ImplicitReturnExpression(TyExpression { return_type, .. }) => {
                type_engine.get(*return_type)
            }
            TyAstNodeContent::SideEffect(_) => TypeInfo::Tuple(Vec::new()),
        }
    }
}

#[derive(Clone, Debug)]
pub enum TyAstNodeContent {
    Declaration(TyDeclaration),
    Expression(TyExpression),
    ImplicitReturnExpression(TyExpression),
    // a no-op node used for something that just issues a side effect, like an import statement.
    SideEffect(TySideEffect),
}

impl EqWithEngines for TyAstNodeContent {}
impl PartialEqWithEngines for TyAstNodeContent {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        match (self, other) {
            (Self::Declaration(x), Self::Declaration(y)) => x.eq(y, engines),
            (Self::Expression(x), Self::Expression(y)) => x.eq(y, engines),
            (Self::ImplicitReturnExpression(x), Self::ImplicitReturnExpression(y)) => {
                x.eq(y, engines)
            }
            (Self::SideEffect(_), Self::SideEffect(_)) => true,
            _ => false,
        }
    }
}

impl HashWithEngines for TyAstNodeContent {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        use TyAstNodeContent::*;
        std::mem::discriminant(self).hash(state);
        match self {
            Declaration(decl) => {
                decl.hash(state, engines);
            }
            Expression(exp) | ImplicitReturnExpression(exp) => {
                exp.hash(state, engines);
            }
            SideEffect(effect) => {
                effect.hash(state);
            }
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
            SideEffect(_) => ok(vec![], vec![], vec![]),
        }
    }
}

impl GetDeclIdent for TyAstNodeContent {
    fn get_decl_ident(&self) -> Option<Ident> {
        match self {
            TyAstNodeContent::Declaration(decl) => decl.get_decl_ident(),
            TyAstNodeContent::Expression(_expr) => None, //expr.get_decl_ident(),
            TyAstNodeContent::ImplicitReturnExpression(_expr) => None, //expr.get_decl_ident(),
            TyAstNodeContent::SideEffect(_) => None,
        }
    }
}
