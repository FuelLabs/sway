use crate::{
    decl_engine::{DeclMapping, MaterializeConstGenerics, ReplaceDecls},
    engine_threading::*,
    language::{parsed, Visibility, ty::{TyExpression, TyVariableDecl, VariableMutability}},
    semantic_analysis::{
        TypeCheckAnalysis, TypeCheckAnalysisContext, TypeCheckContext, TypeCheckFinalization,
        TypeCheckFinalizationContext,
    },
    type_system::*,
    types::*,
    GenericArgument,
};
use ast_elements::type_parameter::ConstGenericExpr;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::{ident::Ident as BaseIdent, Ident, Named, Span, Spanned};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TyModStatement {
    pub span: Span,
    pub visibility: Visibility,
    pub mod_name: BaseIdent,
}

impl Spanned for TyModStatement {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TyUseStatement {
    pub call_path: Vec<BaseIdent>,
    pub span: Span,
    pub import_type: parsed::ImportType,
    // If `is_relative_to_package_root` is true, then this use statement is a path relative to the
    // project root. For example, if the path is `::X::Y` and occurs in package `P`, then the path
    // refers to the full path `P::X::Y`.
    // If `is_relative_to_package_root` is false, then there are two options:
    // - The path refers to a path relative to the current namespace. For example, if the path is
    //   `X::Y` and it occurs in a module whose path is `P::M`, then the path refers to the full
    //   path `P::M::X::Y`.
    // - The path refers to a path in an external package. For example, the path `X::Y` refers to an
    //   entity `Y` in the external package `X`.
    pub is_relative_to_package_root: bool,
    pub alias: Option<BaseIdent>,
}

impl Spanned for TyUseStatement {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
pub enum TyStatement {
    Let(TyLetBinding),
    Use(TyUseStatement),
    Mod(TyModStatement),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TyLetBinding {
    pub name: Ident,
    pub value: TyExpression,
    pub mutability: VariableMutability,
    pub return_type: TypeId,
    pub type_ascription: GenericArgument,
}

impl Named for TyLetBinding {
    fn name(&self) -> &sway_types::BaseIdent {
        &self.name
    }
}

impl Spanned for TyLetBinding {
    fn span(&self) -> Span {
        self.name.span()
    }
}

impl EqWithEngines for TyStatement {}
impl PartialEqWithEngines for TyStatement {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        match (self, other) {
            (TyStatement::Let(lhs), TyStatement::Let(rhs)) => lhs.eq(rhs, ctx),
            (TyStatement::Use(lhs), TyStatement::Use(rhs)) => lhs == rhs,
            (TyStatement::Mod(lhs), TyStatement::Mod(rhs)) => lhs == rhs,
            _ => false,
        }
    }
}

impl HashWithEngines for TyStatement {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        match self {
            TyStatement::Let(binding) => binding.hash(state, engines),
            TyStatement::Use(stmt) => stmt.hash(state),
            TyStatement::Mod(stmt) => stmt.hash(state),
        }
    }
}

impl SubstTypes for TyStatement {
    fn subst_inner(&mut self, ctx: &SubstTypesContext) -> HasChanges {
        match self {
            TyStatement::Let(binding) => binding.subst(ctx),
            TyStatement::Use(_) | TyStatement::Mod(_) => HasChanges::No,
        }
    }
}

impl ReplaceDecls for TyStatement {
    fn replace_decls_inner(
        &mut self,
        decl_mapping: &DeclMapping,
        handler: &Handler,
        ctx: &mut TypeCheckContext,
    ) -> Result<bool, ErrorEmitted> {
        match self {
            TyStatement::Let(binding) => binding.value.replace_decls(decl_mapping, handler, ctx),
            TyStatement::Use(_) | TyStatement::Mod(_) => Ok(false),
        }
    }
}

impl TypeCheckAnalysis for TyStatement {
    fn type_check_analyze(
        &self,
        handler: &Handler,
        ctx: &mut TypeCheckAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        match self {
            TyStatement::Let(binding) => binding.value.type_check_analyze(handler, ctx)?,
            TyStatement::Use(_) | TyStatement::Mod(_) => {}
        }
        Ok(())
    }
}

impl TypeCheckFinalization for TyStatement {
    fn type_check_finalize(
        &mut self,
        handler: &Handler,
        ctx: &mut TypeCheckFinalizationContext,
    ) -> Result<(), ErrorEmitted> {
        match self {
            TyStatement::Let(binding) => binding.value.type_check_finalize(handler, ctx)?,
            TyStatement::Use(_) | TyStatement::Mod(_) => {}
        }
        Ok(())
    }
}

impl CollectTypesMetadata for TyStatement {
    fn collect_types_metadata(
        &self,
        handler: &Handler,
        ctx: &mut CollectTypesMetadataContext,
    ) -> Result<Vec<TypeMetadata>, ErrorEmitted> {
        match self {
            TyStatement::Let(binding) => {
                let mut metadata = binding.value.collect_types_metadata(handler, ctx)?;
                metadata.append(
                    &mut binding
                        .type_ascription
                        .type_id()
                        .collect_types_metadata(handler, ctx)?,
                );
                Ok(metadata)
            }
            TyStatement::Use(_) | TyStatement::Mod(_) => Ok(vec![]),
        }
    }
}

impl MaterializeConstGenerics for TyStatement {
    fn materialize_const_generics(
        &mut self,
        engines: &Engines,
        handler: &Handler,
        name: &str,
        value: &TyExpression,
    ) -> Result<(), ErrorEmitted> {
        match self {
            TyStatement::Let(binding) => {
                binding
                    .value
                    .materialize_const_generics(engines, handler, name, value)?;
                binding
                    .return_type
                    .materialize_const_generics(engines, handler, name, value)?;
                match &mut binding.type_ascription {
                    GenericArgument::Type(arg) => arg
                        .type_id
                        .materialize_const_generics(engines, handler, name, value)?,
                    GenericArgument::Const(arg) => {
                        if matches!(
                            &arg.expr,
                            ConstGenericExpr::AmbiguousVariableExpression { ident, .. }
                                if ident.as_str() == name
                        ) {
                            arg.expr = ConstGenericExpr::from_ty_expression(handler, value)?;
                        }
                    }
                }
                Ok(())
            }
            TyStatement::Use(_) | TyStatement::Mod(_) => Ok(()),
        }
    }
}

impl EqWithEngines for TyLetBinding {}
impl PartialEqWithEngines for TyLetBinding {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        let type_engine = ctx.engines().te();
        self.name == other.name
            && self.value.eq(&other.value, ctx)
            && self.mutability == other.mutability
            && type_engine
                .get(self.return_type)
                .eq(&type_engine.get(other.return_type), ctx)
            && self.type_ascription.eq(&other.type_ascription, ctx)
    }
}

impl HashWithEngines for TyLetBinding {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        let TyLetBinding {
            name,
            value,
            mutability,
            return_type,
            type_ascription,
        } = self;
        let type_engine = engines.te();
        name.hash(state);
        value.hash(state, engines);
        type_engine.get(*return_type).hash(state, engines);
        type_ascription.hash(state, engines);
        mutability.hash(state);
    }
}

impl SubstTypes for TyLetBinding {
    fn subst_inner(&mut self, ctx: &SubstTypesContext) -> HasChanges {
        self.return_type.subst(ctx);
        self.type_ascription.subst(ctx);
        self.value.subst(ctx)
    }
}

impl From<TyVariableDecl> for TyLetBinding {
    fn from(decl: TyVariableDecl) -> Self {
        let TyVariableDecl {
            name,
            body,
            mutability,
            return_type,
            type_ascription,
        } = decl;
        TyLetBinding {
            name,
            value: body,
            mutability,
            return_type,
            type_ascription,
        }
    }
}

impl TyLetBinding {
    pub fn to_variable_decl(&self) -> TyVariableDecl {
        TyVariableDecl {
            name: self.name.clone(),
            body: self.value.clone(),
            mutability: self.mutability,
            return_type: self.return_type,
            type_ascription: self.type_ascription.clone(),
        }
    }
}
