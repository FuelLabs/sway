use std::{
    fmt::{self, Debug},
    hash::{Hash, Hasher},
};

use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::{Ident, Span};

use crate::{
    decl_engine::*,
    engine_threading::*,
    language::{parsed::TreeType, ty::*, Visibility},
    semantic_analysis::{TypeCheckContext, TypeCheckFinalization, TypeCheckFinalizationContext},
    transform::{AllowDeprecatedState, AttributeKind},
    type_system::*,
    types::*,
};

pub trait GetDeclIdent {
    fn get_decl_ident(&self) -> Option<Ident>;
}

#[derive(Clone, Debug)]
pub struct TyAstNode {
    pub content: TyAstNodeContent,
    pub span: Span,
}

impl EqWithEngines for TyAstNode {}
impl PartialEqWithEngines for TyAstNode {
    fn eq(&self, other: &Self, engines: &Engines) -> bool {
        self.content.eq(&other.content, engines)
    }
}

impl HashWithEngines for TyAstNode {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        let TyAstNode {
            content,
            // the span is not hashed because it isn't relevant/a reliable
            // source of obj v. obj distinction
            span: _,
        } = self;
        content.hash(state, engines);
    }
}

impl DebugWithEngines for TyAstNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        use TyAstNodeContent::*;
        match &self.content {
            Declaration(typed_decl) => DebugWithEngines::fmt(typed_decl, f, engines),
            Expression(exp) => DebugWithEngines::fmt(exp, f, engines),
            ImplicitReturnExpression(exp) => write!(f, "return {:?}", engines.help_out(exp)),
            SideEffect(_) => f.write_str(""),
            Error(_, _) => f.write_str("error"),
        }
    }
}

impl SubstTypes for TyAstNode {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: &Engines) {
        match self.content {
            TyAstNodeContent::ImplicitReturnExpression(ref mut exp) => {
                exp.subst(type_mapping, engines)
            }
            TyAstNodeContent::Declaration(ref mut decl) => decl.subst(type_mapping, engines),
            TyAstNodeContent::Expression(ref mut expr) => expr.subst(type_mapping, engines),
            TyAstNodeContent::SideEffect(_) => (),
            TyAstNodeContent::Error(_, _) => (),
        }
    }
}

impl ReplaceSelfType for TyAstNode {
    fn replace_self_type(&mut self, engines: &Engines, self_type: TypeId) {
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
            TyAstNodeContent::Error(_, _) => (),
        }
    }
}

impl ReplaceDecls for TyAstNode {
    fn replace_decls_inner(
        &mut self,
        decl_mapping: &DeclMapping,
        handler: &Handler,
        ctx: &mut TypeCheckContext,
    ) -> Result<(), ErrorEmitted> {
        match self.content {
            TyAstNodeContent::ImplicitReturnExpression(ref mut exp) => {
                exp.replace_decls(decl_mapping, handler, ctx)
            }
            TyAstNodeContent::Declaration(TyDecl::VariableDecl(ref mut decl)) => {
                decl.body.replace_decls(decl_mapping, handler, ctx)
            }
            TyAstNodeContent::Declaration(_) => Ok(()),
            TyAstNodeContent::Expression(ref mut expr) => {
                expr.replace_decls(decl_mapping, handler, ctx)
            }
            TyAstNodeContent::SideEffect(_) => Ok(()),
            TyAstNodeContent::Error(_, _) => Ok(()),
        }
    }
}

impl UpdateConstantExpression for TyAstNode {
    fn update_constant_expression(&mut self, engines: &Engines, implementing_type: &TyDecl) {
        match self.content {
            TyAstNodeContent::ImplicitReturnExpression(ref mut expr) => {
                expr.update_constant_expression(engines, implementing_type)
            }
            TyAstNodeContent::Declaration(_) => {}
            TyAstNodeContent::Expression(ref mut expr) => {
                expr.update_constant_expression(engines, implementing_type)
            }
            TyAstNodeContent::SideEffect(_) => (),
            TyAstNodeContent::Error(_, _) => (),
        }
    }
}

impl TypeCheckFinalization for TyAstNode {
    fn type_check_finalize(
        &mut self,
        handler: &Handler,
        ctx: &mut TypeCheckFinalizationContext,
    ) -> Result<(), ErrorEmitted> {
        self.content.type_check_finalize(handler, ctx)
    }
}

impl CollectTypesMetadata for TyAstNode {
    fn collect_types_metadata(
        &self,
        handler: &Handler,
        ctx: &mut CollectTypesMetadataContext,
    ) -> Result<Vec<TypeMetadata>, ErrorEmitted> {
        self.content.collect_types_metadata(handler, ctx)
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
            Error(_, _) => false,
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
            TyAstNodeContent::Declaration(TyDecl::VariableDecl(decl)) => {
                decl.body.gather_return_statements()
            }
            TyAstNodeContent::Expression(exp) => exp.gather_return_statements(),
            TyAstNodeContent::Error(_, _) => vec![],
            TyAstNodeContent::SideEffect(_) | TyAstNodeContent::Declaration(_) => vec![],
        }
    }

    /// Returns `true` if this AST node will be exported in a library, i.e. it is a public declaration.
    pub(crate) fn is_public(&self, decl_engine: &DeclEngine) -> bool {
        match &self.content {
            TyAstNodeContent::Declaration(decl) => decl.visibility(decl_engine).is_public(),
            TyAstNodeContent::Expression(_)
            | TyAstNodeContent::SideEffect(_)
            | TyAstNodeContent::Error(_, _)
            | TyAstNodeContent::ImplicitReturnExpression(_) => false,
        }
    }

    /// Check to see if this node is a function declaration with generic type parameters.
    pub(crate) fn is_generic_function(&self, decl_engine: &DeclEngine) -> bool {
        match &self {
            TyAstNode {
                span: _,
                content:
                    TyAstNodeContent::Declaration(TyDecl::FunctionDecl(FunctionDecl {
                        decl_id, ..
                    })),
                ..
            } => {
                let TyFunctionDecl {
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
                    TyAstNodeContent::Declaration(TyDecl::FunctionDecl(FunctionDecl {
                        decl_id, ..
                    })),
                ..
            } => {
                let TyFunctionDecl { attributes, .. } = decl_engine.get_function(decl_id);
                attributes.contains_key(&AttributeKind::Test)
            }
            _ => false,
        }
    }

    pub(crate) fn is_entry_point(&self, decl_engine: &DeclEngine, tree_type: &TreeType) -> bool {
        match tree_type {
            TreeType::Predicate | TreeType::Script => {
                // Predicates and scripts have main and test functions as entry points.
                match self {
                    TyAstNode {
                        span: _,
                        content:
                            TyAstNodeContent::Declaration(TyDecl::FunctionDecl(FunctionDecl {
                                decl_id,
                                ..
                            })),
                        ..
                    } => {
                        let decl = decl_engine.get_function(decl_id);
                        decl.is_entry()
                    }
                    _ => false,
                }
            }
            TreeType::Contract | TreeType::Library { .. } => match self {
                TyAstNode {
                    content:
                        TyAstNodeContent::Declaration(TyDecl::FunctionDecl(FunctionDecl {
                            decl_id,
                            decl_span: _,
                            ..
                        })),
                    ..
                } => {
                    let decl = decl_engine.get_function(decl_id);
                    decl.visibility == Visibility::Public || decl.is_test()
                }
                TyAstNode {
                    content:
                        TyAstNodeContent::Declaration(TyDecl::TraitDecl(TraitDecl {
                            decl_id,
                            decl_span: _,
                            ..
                        })),
                    ..
                } => decl_engine.get_trait(decl_id).visibility.is_public(),
                TyAstNode {
                    content:
                        TyAstNodeContent::Declaration(TyDecl::StructDecl(StructDecl {
                            decl_id, ..
                        })),
                    ..
                } => {
                    let struct_decl = decl_engine.get_struct(decl_id);
                    struct_decl.visibility == Visibility::Public
                }
                TyAstNode {
                    content: TyAstNodeContent::Declaration(TyDecl::ImplTrait { .. }),
                    ..
                } => true,
                TyAstNode {
                    content:
                        TyAstNodeContent::Declaration(TyDecl::ConstantDecl(ConstantDecl {
                            decl_id,
                            decl_span: _,
                            ..
                        })),
                    ..
                } => {
                    let decl = decl_engine.get_constant(decl_id);
                    decl.visibility.is_public()
                }
                TyAstNode {
                    content:
                        TyAstNodeContent::Declaration(TyDecl::TypeAliasDecl(TypeAliasDecl {
                            decl_id,
                            ..
                        })),
                    ..
                } => {
                    let decl = decl_engine.get_type_alias(decl_id);
                    decl.visibility.is_public()
                }
                _ => false,
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
            TyAstNodeContent::Error(_, error) => TypeInfo::ErrorRecovery(*error),
        }
    }

    pub(crate) fn check_deprecated(
        &self,
        engines: &Engines,
        handler: &Handler,
        allow_deprecated: &mut AllowDeprecatedState,
    ) {
        match &self.content {
            TyAstNodeContent::Declaration(node) => match node {
                TyDecl::VariableDecl(decl) => {
                    decl.body
                        .check_deprecated(engines, handler, allow_deprecated);
                }
                TyDecl::ConstantDecl(decl) => {
                    let decl = engines.de().get(&decl.decl_id);
                    if let Some(value) = decl.value {
                        value.check_deprecated(engines, handler, allow_deprecated);
                    }
                }
                TyDecl::TraitTypeDecl(_) => {}
                TyDecl::FunctionDecl(decl) => {
                    let decl = engines.de().get(&decl.decl_id);
                    let token = allow_deprecated.enter(decl.attributes);
                    for node in decl.body.contents.iter() {
                        node.check_deprecated(engines, handler, allow_deprecated);
                    }
                    allow_deprecated.exit(token);
                }
                TyDecl::ImplTrait(decl) => {
                    let decl = engines.de().get(&decl.decl_id);
                    for item in decl.items.iter() {
                        match item {
                            TyTraitItem::Fn(item) => {
                                let decl = engines.de().get(item.id());
                                let token = allow_deprecated.enter(decl.attributes);
                                for node in decl.body.contents.iter() {
                                    node.check_deprecated(engines, handler, allow_deprecated);
                                }
                                allow_deprecated.exit(token);
                            }
                            TyTraitItem::Constant(item) => {
                                let decl = engines.de().get(item.id());
                                if let Some(expr) = decl.value.as_ref() {
                                    expr.check_deprecated(engines, handler, allow_deprecated);
                                }
                            }
                            TyTraitItem::Type(_) => {}
                        }
                    }
                }
                TyDecl::AbiDecl(_)
                | TyDecl::GenericTypeForFunctionScope(_)
                | TyDecl::ErrorRecovery(_, _)
                | TyDecl::StorageDecl(_)
                | TyDecl::TraitDecl(_)
                | TyDecl::StructDecl(_)
                | TyDecl::EnumDecl(_)
                | TyDecl::EnumVariantDecl(_)
                | TyDecl::TypeAliasDecl(_) => {}
            },
            TyAstNodeContent::Expression(node) => {
                node.check_deprecated(engines, handler, allow_deprecated);
            }
            TyAstNodeContent::ImplicitReturnExpression(node) => {
                node.check_deprecated(engines, handler, allow_deprecated);
            }
            TyAstNodeContent::SideEffect(_) | TyAstNodeContent::Error(_, _) => {}
        }
    }
}

#[derive(Clone, Debug)]
pub enum TyAstNodeContent {
    Declaration(TyDecl),
    Expression(TyExpression),
    ImplicitReturnExpression(TyExpression),
    // a no-op node used for something that just issues a side effect, like an import statement.
    SideEffect(TySideEffect),
    Error(Box<[Span]>, ErrorEmitted),
}

impl EqWithEngines for TyAstNodeContent {}
impl PartialEqWithEngines for TyAstNodeContent {
    fn eq(&self, other: &Self, engines: &Engines) -> bool {
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
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
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
            Error(_, _) => {}
        }
    }
}

impl TypeCheckFinalization for TyAstNodeContent {
    fn type_check_finalize(
        &mut self,
        handler: &Handler,
        ctx: &mut TypeCheckFinalizationContext,
    ) -> Result<(), ErrorEmitted> {
        match self {
            TyAstNodeContent::Declaration(node) => node.type_check_finalize(handler, ctx)?,
            TyAstNodeContent::Expression(node) => node.type_check_finalize(handler, ctx)?,
            TyAstNodeContent::ImplicitReturnExpression(node) => {
                node.type_check_finalize(handler, ctx)?
            }
            TyAstNodeContent::SideEffect(_) => {}
            TyAstNodeContent::Error(_, _) => {}
        }
        Ok(())
    }
}

impl CollectTypesMetadata for TyAstNodeContent {
    fn collect_types_metadata(
        &self,
        handler: &Handler,
        ctx: &mut CollectTypesMetadataContext,
    ) -> Result<Vec<TypeMetadata>, ErrorEmitted> {
        use TyAstNodeContent::*;
        match self {
            Declaration(decl) => decl.collect_types_metadata(handler, ctx),
            Expression(expr) => expr.collect_types_metadata(handler, ctx),
            ImplicitReturnExpression(expr) => expr.collect_types_metadata(handler, ctx),
            SideEffect(_) => Ok(vec![]),
            Error(_, _) => Ok(vec![]),
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
            TyAstNodeContent::Error(_, _) => None,
        }
    }
}
