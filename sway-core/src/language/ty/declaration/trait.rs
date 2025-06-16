use crate::{
    decl_engine::{
        DeclEngineReplace, DeclId, DeclRefConstant, DeclRefFunction, DeclRefTraitFn, DeclRefTraitType, MaterializeConstGenerics, ReplaceFunctionImplementingType
    },
    engine_threading::*,
    has_changes,
    language::{
        parsed::{self, TraitDeclaration},
        ty::{TyConstGenericDecl, TyDecl, TyDeclParsedType},
        CallPath, Visibility,
    },
    semantic_analysis::{
        TypeCheckAnalysis, TypeCheckAnalysisContext, TypeCheckFinalization,
        TypeCheckFinalizationContext,
    },
    transform,
    type_system::*,
};
use monomorphization::MonomorphizeHelper;
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    hash::{Hash, Hasher},
};
use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::{Ident, Named, Span, Spanned};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TyTraitDecl {
    pub name: Ident,
    pub type_parameters: Vec<TypeParameter>,
    pub self_type: TypeParameter,
    pub interface_surface: Vec<TyTraitInterfaceItem>,
    pub items: Vec<TyTraitItem>,
    pub supertraits: Vec<parsed::Supertrait>,
    pub visibility: Visibility,
    pub attributes: transform::Attributes,
    pub call_path: CallPath,
    pub span: Span,
}

impl TyDeclParsedType for TyTraitDecl {
    type ParsedType = TraitDeclaration;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TyTraitInterfaceItem {
    TraitFn(DeclRefTraitFn),
    Constant(DeclRefConstant),
    Type(DeclRefTraitType),
}

impl DisplayWithEngines for TyTraitInterfaceItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        write!(f, "{:?}", engines.help_out(self))
    }
}

impl DebugWithEngines for TyTraitInterfaceItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        write!(
            f,
            "TyTraitItem {}",
            match self {
                TyTraitInterfaceItem::TraitFn(fn_ref) => format!(
                    "fn {:?}",
                    engines.help_out(&*engines.de().get_trait_fn(fn_ref))
                ),
                TyTraitInterfaceItem::Constant(const_ref) => format!(
                    "const {:?}",
                    engines.help_out(&*engines.de().get_constant(const_ref))
                ),
                TyTraitInterfaceItem::Type(type_ref) => format!(
                    "type {:?}",
                    engines.help_out(&*engines.de().get_type(type_ref))
                ),
            }
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TyTraitItem {
    Fn(DeclRefFunction),
    Constant(DeclRefConstant),
    Type(DeclRefTraitType),
}

impl DisplayWithEngines for TyTraitItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        write!(f, "{:?}", engines.help_out(self))
    }
}

impl DebugWithEngines for TyTraitItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        write!(
            f,
            "TyTraitItem {}",
            match self {
                TyTraitItem::Fn(fn_ref) => format!(
                    "fn {:?}",
                    engines.help_out(&*engines.de().get_function(fn_ref))
                ),
                TyTraitItem::Constant(const_ref) => format!(
                    "const {:?}",
                    engines.help_out(&*engines.de().get_constant(const_ref))
                ),
                TyTraitItem::Type(type_ref) => format!(
                    "type {:?}",
                    engines.help_out(&*engines.de().get_type(type_ref))
                ),
            }
        )
    }
}

impl Named for TyTraitDecl {
    fn name(&self) -> &Ident {
        &self.name
    }
}

impl Spanned for TyTraitDecl {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl EqWithEngines for TyTraitDecl {}
impl PartialEqWithEngines for TyTraitDecl {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.name == other.name
            && self.type_parameters.eq(&other.type_parameters, ctx)
            && self.interface_surface.eq(&other.interface_surface, ctx)
            && self.items.eq(&other.items, ctx)
            && self.supertraits.eq(&other.supertraits, ctx)
            && self.visibility == other.visibility
    }
}

impl HashWithEngines for TyTraitDecl {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        let TyTraitDecl {
            name,
            type_parameters,
            self_type,
            interface_surface,
            items,
            supertraits,
            visibility,
            // these fields are not hashed because they aren't relevant/a
            // reliable source of obj v. obj distinction
            attributes: _,
            span: _,
            call_path: _,
        } = self;
        name.hash(state);
        type_parameters.hash(state, engines);
        self_type.hash(state, engines);
        interface_surface.hash(state, engines);
        items.hash(state, engines);
        supertraits.hash(state, engines);
        visibility.hash(state);
    }
}

impl MaterializeConstGenerics for TyTraitDecl {
    fn materialize_const_generics(
        &mut self,
        _engines: &Engines,
        _handler: &Handler,
        _name: DeclId<TyConstGenericDecl>,
        _value: &crate::language::ty::TyExpression,
    ) -> Result<(), ErrorEmitted> {
        Ok(())
    }
}

impl EqWithEngines for TyTraitInterfaceItem {}
impl PartialEqWithEngines for TyTraitInterfaceItem {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        match (self, other) {
            (TyTraitInterfaceItem::TraitFn(id), TyTraitInterfaceItem::TraitFn(other_id)) => {
                id.eq(other_id, ctx)
            }
            (TyTraitInterfaceItem::Constant(id), TyTraitInterfaceItem::Constant(other_id)) => {
                id.eq(other_id, ctx)
            }
            _ => false,
        }
    }
}

impl EqWithEngines for TyTraitItem {}
impl PartialEqWithEngines for TyTraitItem {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        match (self, other) {
            (TyTraitItem::Fn(id), TyTraitItem::Fn(other_id)) => id.eq(other_id, ctx),
            (TyTraitItem::Constant(id), TyTraitItem::Constant(other_id)) => id.eq(other_id, ctx),
            _ => false,
        }
    }
}

impl HashWithEngines for TyTraitInterfaceItem {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        match self {
            TyTraitInterfaceItem::TraitFn(fn_decl) => fn_decl.hash(state, engines),
            TyTraitInterfaceItem::Constant(const_decl) => const_decl.hash(state, engines),
            TyTraitInterfaceItem::Type(type_decl) => type_decl.hash(state, engines),
        }
    }
}

impl HashWithEngines for TyTraitItem {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        match self {
            TyTraitItem::Fn(fn_decl) => fn_decl.hash(state, engines),
            TyTraitItem::Constant(const_decl) => const_decl.hash(state, engines),
            TyTraitItem::Type(type_decl) => type_decl.hash(state, engines),
        }
    }
}

impl TypeCheckAnalysis for TyTraitItem {
    fn type_check_analyze(
        &self,
        handler: &Handler,
        ctx: &mut TypeCheckAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        let decl_engine = ctx.engines.de();

        match self {
            TyTraitItem::Fn(node) => {
                node.type_check_analyze(handler, ctx)?;
            }
            TyTraitItem::Constant(node) => {
                let item_const = decl_engine.get_constant(node);
                item_const.type_check_analyze(handler, ctx)?;
            }
            TyTraitItem::Type(node) => {
                let item_type = decl_engine.get_type(node);
                item_type.type_check_analyze(handler, ctx)?;
            }
        }

        Ok(())
    }
}

impl TypeCheckFinalization for TyTraitItem {
    fn type_check_finalize(
        &mut self,
        handler: &Handler,
        ctx: &mut TypeCheckFinalizationContext,
    ) -> Result<(), ErrorEmitted> {
        let decl_engine = ctx.engines.de();
        match self {
            TyTraitItem::Fn(node) => {
                let mut item_fn = (*decl_engine.get_function(node)).clone();
                item_fn.type_check_finalize(handler, ctx)?;
                decl_engine.replace(*node.id(), item_fn);
            }
            TyTraitItem::Constant(node) => {
                let mut item_const = (*decl_engine.get_constant(node)).clone();
                item_const.type_check_finalize(handler, ctx)?;
                decl_engine.replace(*node.id(), item_const);
            }
            TyTraitItem::Type(_node) => {
                // Nothing to finalize
            }
        }
        Ok(())
    }
}

impl Spanned for TyTraitItem {
    fn span(&self) -> Span {
        match self {
            TyTraitItem::Fn(fn_decl) => fn_decl.span(),
            TyTraitItem::Constant(const_decl) => const_decl.span(),
            TyTraitItem::Type(type_decl) => type_decl.span(),
        }
    }
}

impl SubstTypes for TyTraitDecl {
    fn subst_inner(&mut self, ctx: &SubstTypesContext) -> HasChanges {
        has_changes! {
            self.type_parameters.subst(ctx);
            self.interface_surface
                .iter_mut()
                .fold(HasChanges::No, |has_changes, item| match item {
                    TyTraitInterfaceItem::TraitFn(item_ref) => {
                        if let Some(new_item_ref) = item_ref
                            .clone()
                            .subst_types_and_insert_new_with_parent(ctx) {
                            item_ref.replace_id(*new_item_ref.id());
                            HasChanges::Yes
                        } else {
                            HasChanges::No
                        }
                    }
                    TyTraitInterfaceItem::Constant(decl_ref) => {
                        if let Some(new_decl_ref) = decl_ref
                            .clone()
                            .subst_types_and_insert_new(ctx) {
                            decl_ref.replace_id(*new_decl_ref.id());
                            HasChanges::Yes
                        } else{
                            HasChanges::No
                        }
                    }
                    TyTraitInterfaceItem::Type(decl_ref) => {
                        if let Some(new_decl_ref) = decl_ref
                            .clone()
                            .subst_types_and_insert_new(ctx) {
                            decl_ref.replace_id(*new_decl_ref.id());
                            HasChanges::Yes
                        } else{
                            HasChanges::No
                        }
                    }
                } | has_changes);
            self.items.iter_mut().fold(HasChanges::No, |has_changes, item| match item {
                TyTraitItem::Fn(item_ref) => {
                    if let Some(new_item_ref) = item_ref
                        .clone()
                        .subst_types_and_insert_new_with_parent(ctx)
                    {
                        item_ref.replace_id(*new_item_ref.id());
                        HasChanges::Yes
                    } else {
                        HasChanges::No
                    }
                }
                TyTraitItem::Constant(item_ref) => {
                    if let Some(new_decl_ref) = item_ref
                        .clone()
                        .subst_types_and_insert_new_with_parent(ctx)
                    {
                        item_ref.replace_id(*new_decl_ref.id());
                        HasChanges::Yes
                    } else {
                        HasChanges::No
                    }
                }
                TyTraitItem::Type(item_ref) => {
                    if let Some(new_decl_ref) = item_ref
                        .clone()
                        .subst_types_and_insert_new_with_parent(ctx)
                    {
                        item_ref.replace_id(*new_decl_ref.id());
                        HasChanges::Yes
                    } else {
                        HasChanges::No
                    }
                }
            } | has_changes);
        }
    }
}

impl SubstTypes for TyTraitItem {
    fn subst_inner(&mut self, ctx: &SubstTypesContext) -> HasChanges {
        match self {
            TyTraitItem::Fn(fn_decl) => fn_decl.subst(ctx),
            TyTraitItem::Constant(const_decl) => const_decl.subst(ctx),
            TyTraitItem::Type(type_decl) => type_decl.subst(ctx),
        }
    }
}

impl ReplaceFunctionImplementingType for TyTraitItem {
    fn replace_implementing_type(&mut self, engines: &Engines, implementing_type: TyDecl) {
        match self {
            TyTraitItem::Fn(decl_ref) => {
                decl_ref.replace_implementing_type(engines, implementing_type)
            }
            TyTraitItem::Constant(_decl_ref) => {
                // ignore, only needed for functions
            }
            TyTraitItem::Type(_decl_ref) => {
                // ignore, only needed for functions
            }
        }
    }
}

impl MonomorphizeHelper for TyTraitDecl {
    fn name(&self) -> &Ident {
        &self.name
    }

    fn type_parameters(&self) -> &[TypeParameter] {
        &self.type_parameters
    }

    fn has_self_type_param(&self) -> bool {
        true
    }
}
