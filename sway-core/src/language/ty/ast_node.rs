use std::{
    fmt::{self, Debug},
    hash::{Hash, Hasher},
};

use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::{Ident, Span};

use crate::{
    decl_engine::*,
    engine_threading::*,
    language::ty::*,
    semantic_analysis::{
        TypeCheckAnalysis, TypeCheckAnalysisContext, TypeCheckContext, TypeCheckFinalization,
        TypeCheckFinalizationContext,
    },
    transform::{AllowDeprecatedState, AttributeKind},
    type_system::*,
    types::*,
};

pub trait GetDeclIdent {
    fn get_decl_ident(&self, engines: &Engines) -> Option<Ident>;
}

#[derive(Clone, Debug)]
pub struct TyAstNode {
    pub content: TyAstNodeContent,
    pub span: Span,
}

impl EqWithEngines for TyAstNode {}
impl PartialEqWithEngines for TyAstNode {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.content.eq(&other.content, ctx)
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
            SideEffect(_) => f.write_str(""),
            Error(_, _) => f.write_str("error"),
        }
    }
}

impl SubstTypes for TyAstNode {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: &Engines) -> HasChanges {
        match self.content {
            TyAstNodeContent::Declaration(ref mut decl) => decl.subst(type_mapping, engines),
            TyAstNodeContent::Expression(ref mut expr) => expr.subst(type_mapping, engines),
            TyAstNodeContent::SideEffect(_) | TyAstNodeContent::Error(_, _) => HasChanges::No,
        }
    }
}

impl ReplaceDecls for TyAstNode {
    fn replace_decls_inner(
        &mut self,
        decl_mapping: &DeclMapping,
        handler: &Handler,
        ctx: &mut TypeCheckContext,
    ) -> Result<bool, ErrorEmitted> {
        match self.content {
            TyAstNodeContent::Declaration(TyDecl::VariableDecl(ref mut decl)) => {
                decl.body.replace_decls(decl_mapping, handler, ctx)
            }
            TyAstNodeContent::Declaration(_) => Ok(false),
            TyAstNodeContent::Expression(ref mut expr) => {
                expr.replace_decls(decl_mapping, handler, ctx)
            }
            TyAstNodeContent::SideEffect(_) => Ok(false),
            TyAstNodeContent::Error(_, _) => Ok(false),
        }
    }
}

impl UpdateConstantExpression for TyAstNode {
    fn update_constant_expression(&mut self, engines: &Engines, implementing_type: &TyDecl) {
        match self.content {
            TyAstNodeContent::Declaration(_) => {}
            TyAstNodeContent::Expression(ref mut expr) => {
                expr.update_constant_expression(engines, implementing_type)
            }
            TyAstNodeContent::SideEffect(_) => (),
            TyAstNodeContent::Error(_, _) => (),
        }
    }
}

impl TypeCheckAnalysis for TyAstNode {
    fn type_check_analyze(
        &self,
        handler: &Handler,
        ctx: &mut TypeCheckAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        self.content.type_check_analyze(handler, ctx)
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

impl GetDeclIdent for TyAstNode {
    fn get_decl_ident(&self, engines: &Engines) -> Option<Ident> {
        self.content.get_decl_ident(engines)
    }
}

impl TyAstNode {
    /// Returns `true` if this AST node will be exported in a library, i.e. it is a public declaration.
    pub(crate) fn is_public(&self, decl_engine: &DeclEngine) -> bool {
        match &self.content {
            TyAstNodeContent::Declaration(decl) => decl.visibility(decl_engine).is_public(),
            TyAstNodeContent::Expression(_)
            | TyAstNodeContent::SideEffect(_)
            | TyAstNodeContent::Error(_, _) => false,
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
                let fn_decl = decl_engine.get_function(decl_id);
                let TyFunctionDecl {
                    type_parameters, ..
                } = &*fn_decl;
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
                let fn_decl = decl_engine.get_function(decl_id);
                let TyFunctionDecl { attributes, .. } = &*fn_decl;
                attributes.contains_key(&AttributeKind::Test)
            }
            _ => false,
        }
    }

    pub(crate) fn type_info(&self, type_engine: &TypeEngine) -> TypeInfo {
        // return statement should be ()
        match &self.content {
            TyAstNodeContent::Declaration(_) => TypeInfo::Tuple(Vec::new()),
            TyAstNodeContent::Expression(TyExpression { return_type, .. }) => {
                (*type_engine.get(*return_type)).clone()
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
                    if let Some(value) = &decl.value {
                        value.check_deprecated(engines, handler, allow_deprecated);
                    }
                }
                TyDecl::TraitTypeDecl(_) => {}
                TyDecl::FunctionDecl(decl) => {
                    let decl = engines.de().get(&decl.decl_id);
                    let token = allow_deprecated.enter(decl.attributes.clone());
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
                                let token = allow_deprecated.enter(decl.attributes.clone());
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
            TyAstNodeContent::SideEffect(_) | TyAstNodeContent::Error(_, _) => {}
        }
    }

    pub(crate) fn check_recursive(
        &self,
        engines: &Engines,
        handler: &Handler,
    ) -> Result<(), ErrorEmitted> {
        handler.scope(|handler| {
            match &self.content {
                TyAstNodeContent::Declaration(node) => match node {
                    TyDecl::VariableDecl(_decl) => {}
                    TyDecl::ConstantDecl(_decl) => {}
                    TyDecl::TraitTypeDecl(_) => {}
                    TyDecl::FunctionDecl(decl) => {
                        let fn_decl_id = decl.decl_id;
                        let mut ctx = TypeCheckAnalysisContext::new(engines);
                        let _ = fn_decl_id.type_check_analyze(handler, &mut ctx);
                        let _ = ctx.check_recursive_calls(handler);
                    }
                    TyDecl::ImplTrait(decl) => {
                        let decl = engines.de().get(&decl.decl_id);
                        for item in decl.items.iter() {
                            let mut ctx = TypeCheckAnalysisContext::new(engines);
                            let _ = item.type_check_analyze(handler, &mut ctx);
                            let _ = ctx.check_recursive_calls(handler);
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
                TyAstNodeContent::Expression(_node) => {}
                TyAstNodeContent::SideEffect(_) | TyAstNodeContent::Error(_, _) => {}
            };
            Ok(())
        })
    }

    pub fn contract_fns(&self, engines: &Engines) -> Vec<DeclRefFunction> {
        let mut fns = vec![];

        if let TyAstNodeContent::Declaration(TyDecl::ImplTrait(decl)) = &self.content {
            let decl = engines.de().get(&decl.decl_id);
            if decl.is_impl_contract(engines.te()) {
                for item in &decl.items {
                    if let TyTraitItem::Fn(f) = item {
                        fns.push(f.clone());
                    }
                }
            }
        }

        fns
    }
}

#[derive(Clone, Debug)]
pub enum TyAstNodeContent {
    Declaration(TyDecl),
    Expression(TyExpression),
    // a no-op node used for something that just issues a side effect, like an import statement.
    SideEffect(TySideEffect),
    Error(Box<[Span]>, ErrorEmitted),
}

impl EqWithEngines for TyAstNodeContent {}
impl PartialEqWithEngines for TyAstNodeContent {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        match (self, other) {
            (Self::Declaration(x), Self::Declaration(y)) => x.eq(y, ctx),
            (Self::Expression(x), Self::Expression(y)) => x.eq(y, ctx),
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
            Expression(exp) => {
                exp.hash(state, engines);
            }
            SideEffect(effect) => {
                effect.hash(state);
            }
            Error(_, _) => {}
        }
    }
}

impl TypeCheckAnalysis for TyAstNodeContent {
    fn type_check_analyze(
        &self,
        handler: &Handler,
        ctx: &mut TypeCheckAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        match self {
            TyAstNodeContent::Declaration(node) => node.type_check_analyze(handler, ctx)?,
            TyAstNodeContent::Expression(node) => node.type_check_analyze(handler, ctx)?,
            TyAstNodeContent::SideEffect(_) => {}
            TyAstNodeContent::Error(_, _) => {}
        }
        Ok(())
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
            SideEffect(_) => Ok(vec![]),
            Error(_, _) => Ok(vec![]),
        }
    }
}

impl GetDeclIdent for TyAstNodeContent {
    fn get_decl_ident(&self, engines: &Engines) -> Option<Ident> {
        match self {
            TyAstNodeContent::Declaration(decl) => decl.get_decl_ident(engines),
            TyAstNodeContent::Expression(_expr) => None, //expr.get_decl_ident(),
            TyAstNodeContent::SideEffect(_) => None,
            TyAstNodeContent::Error(_, _) => None,
        }
    }
}
