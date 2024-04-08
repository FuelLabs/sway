use std::{
    fmt,
    hash::{Hash, Hasher},
};

use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::{Ident, Named, Span, Spanned};

use crate::{
    decl_engine::{
        AssociatedItemDeclId, DeclEngineInsert, DeclEngineReplace, DeclId, DeclRef,
        DeclRefConstant, DeclRefFunction, DeclRefTraitFn, DeclRefTraitType, InProgressSubstTypes,
        ReplaceFunctionImplementingType,
    },
    engine_threading::*,
    language::{parsed, CallPath, Visibility},
    semantic_analysis::{
        type_check_context::MonomorphizeHelper, TypeCheckAnalysis, TypeCheckAnalysisContext,
        TypeCheckFinalization, TypeCheckFinalizationContext,
    },
    subs, transform,
    type_system::*,
};

use super::{TyConstantDecl, TyDecl, TyFunctionDecl, TyTraitType};

#[derive(Clone, Debug)]
pub struct TyTraitDecl {
    pub name: Ident,
    pub type_parameters: Vec<TypeParameter>,
    pub self_type: TypeParameter,
    pub interface_surface: Vec<TyTraitInterfaceItem>,
    pub items: Vec<TyTraitItem>,
    pub supertraits: Vec<parsed::Supertrait>,
    pub visibility: Visibility,
    pub attributes: transform::AttributesMap,
    pub call_path: CallPath,
    pub span: Span,
}

#[derive(Clone, Debug)]
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

impl SubstTypes for TyTraitInterfaceItem {
    fn subst_inner(&self, type_mapping: &TypeSubstMap, engines: &Engines) -> Option<Self> {
        match self {
            TyTraitInterfaceItem::TraitFn(r) => {
                let r = r
                    .start_subst_types()
                    .subst(type_mapping, engines)?
                    .insert_new_with_parent(engines);
                Some(Self::TraitFn(r))
            }
            TyTraitInterfaceItem::Constant(r) => {
                let r = r
                    .start_subst_types()
                    .subst(type_mapping, engines)?
                    .insert_new(engines);
                Some(Self::Constant(r))
            }
            TyTraitInterfaceItem::Type(r) => {
                let r = r
                    .start_subst_types()
                    .subst(type_mapping, engines)?
                    .insert_new(engines);
                Some(Self::Type(r))
            }
        }
    }
}

pub trait VecTyTraitItemExtensions<'a> {
    fn start_subst_types(&self) -> InProgress<'a>;
}

impl<'a> VecTyTraitItemExtensions<'a> for &'a Vec<TyTraitItem> {
    fn start_subst_types(&self) -> InProgress<'a> {
        InProgress {
            before: self,
            after: None,
        }
    }
}

#[derive(Clone, Debug)]
pub enum TyTraitItem {
    Fn(DeclRefFunction),
    Constant(DeclRefConstant),
    Type(DeclRefTraitType),
}

#[derive(Clone)]
pub enum InProgressTyTraitItem<'a> {
    Fn(InProgressSubstTypes<'a, DeclRef<DeclId<TyFunctionDecl>>, TyFunctionDecl>),
    Constant(InProgressSubstTypes<'a, DeclRef<DeclId<TyConstantDecl>>, TyConstantDecl>),
    Type(InProgressSubstTypes<'a, DeclRef<DeclId<TyTraitType>>, TyTraitType>),
}

#[derive(Clone)]
pub struct InProgress<'a> {
    before: &'a Vec<TyTraitItem>,
    after: Option<Vec<InProgressTyTraitItem<'a>>>,
}

impl<'a> InProgress<'a> {
    pub fn replace(self, engines: &Engines) -> Vec<TyTraitItem> {
        match self.after {
            Some(x) => x
                .into_iter()
                .map(|x| match x {
                    InProgressTyTraitItem::Fn(x) => TyTraitItem::Fn(x.replace(engines)),
                    InProgressTyTraitItem::Constant(x) => TyTraitItem::Constant(x.replace(engines)),
                    InProgressTyTraitItem::Type(x) => TyTraitItem::Type(x.replace(engines)),
                })
                .collect(),
            None => self.before.clone(),
        }
    }

    pub fn insert_new_with_parent(self, engines: &Engines) -> Vec<TyTraitItem> {
        match self.after {
            Some(x) => x
                .into_iter()
                .map(|x| match x {
                    InProgressTyTraitItem::Fn(x) => {
                        TyTraitItem::Fn(x.insert_new_with_parent(engines))
                    }
                    InProgressTyTraitItem::Constant(x) => {
                        TyTraitItem::Constant(x.insert_new_with_parent(engines))
                    }
                    InProgressTyTraitItem::Type(x) => {
                        TyTraitItem::Type(x.insert_new_with_parent(engines))
                    }
                })
                .collect(),
            None => self.before.clone(),
        }
    }
}

impl<'a> SubstTypes for InProgress<'a> {
    fn subst_inner(&self, type_mapping: &TypeSubstMap, engines: &Engines) -> Option<Self> {
        let mut iter = self.before.iter();
        let mut i = -1isize;

        while let Some(item) = iter.next() {
            i += 1;

            let changed = match item {
                TyTraitItem::Fn(r) => match r.start_subst_types().subst(type_mapping, engines) {
                    None => continue,
                    Some(x) => InProgressTyTraitItem::Fn(x),
                },
                TyTraitItem::Constant(r) => {
                    match r.start_subst_types().subst(type_mapping, engines) {
                        None => continue,
                        Some(x) => InProgressTyTraitItem::Constant(x),
                    }
                }
                TyTraitItem::Type(r) => match r.start_subst_types().subst(type_mapping, engines) {
                    None => continue,
                    Some(x) => InProgressTyTraitItem::Type(x),
                },
            };

            let mut new_vec = Vec::with_capacity(self.before.len());
            new_vec.extend(
                self.before
                    .iter()
                    .map(|x| match x {
                        TyTraitItem::Fn(x) => InProgressTyTraitItem::Fn(x.start_subst_types()),
                        TyTraitItem::Constant(x) => {
                            InProgressTyTraitItem::Constant(x.start_subst_types())
                        }
                        TyTraitItem::Type(x) => InProgressTyTraitItem::Type(x.start_subst_types()),
                    })
                    .take(i as usize),
            );
            new_vec.push(changed);

            while let Some(item) = iter.next() {
                let new_item = match item {
                    TyTraitItem::Fn(r) => {
                        match r.start_subst_types().subst(type_mapping, engines) {
                            None => InProgressTyTraitItem::Fn(r.start_subst_types()),
                            Some(x) => InProgressTyTraitItem::Fn(x),
                        }
                    }
                    TyTraitItem::Constant(r) => {
                        match r.start_subst_types().subst(type_mapping, engines) {
                            None => InProgressTyTraitItem::Constant(r.start_subst_types()),
                            Some(x) => InProgressTyTraitItem::Constant(x),
                        }
                    }
                    TyTraitItem::Type(r) => {
                        match r.start_subst_types().subst(type_mapping, engines) {
                            None => InProgressTyTraitItem::Type(r.start_subst_types()),
                            Some(x) => InProgressTyTraitItem::Type(x),
                        }
                    }
                };
                new_vec.push(new_item);
            }

            return Some(InProgress {
                before: &self.before,
                after: Some(new_vec),
            });
        }

        None
    }
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

// impl SubstTypes for TyTraitItem {
//     fn subst_inner(&self, type_mapping: &TypeSubstMap, engines: &Engines) -> Option<Self> {
//         match self {
//             Self::Fn(r) => {
//                 let r = r
//                     .start_subst_types()
//                     .subst(type_mapping, engines)?
//                     .insert_new_with_parent(engines);
//                 Some(Self::Fn(r))
//             }
//             Self::Constant(r) => {
//                 let r = r
//                     .start_subst_types()
//                     .subst(type_mapping, engines)?
//                     .insert_new_with_parent(engines);
//                 Some(Self::Constant(r))
//             }
//             Self::Type(r) => {
//                 let r = r
//                     .start_subst_types()
//                     .subst(type_mapping, engines)?
//                     .insert_new_with_parent(engines);
//                 Some(Self::Type(r))
//             }
//         }
//     }
// }

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
    fn subst_inner(&self, type_mapping: &TypeSubstMap, engines: &Engines) -> Option<Self> {
        let (type_parameters, interface_surface, items) = subs! {
            self.type_parameters,
            self.interface_surface,
            (&self.items).start_subst_types()
        }(type_mapping, engines)?;
        Some(Self {
            type_parameters,
            interface_surface,
            items: items.insert_new_with_parent(engines),
            name: self.name.clone(),
            self_type: self.self_type.clone(),
            supertraits: self.supertraits.clone(),
            visibility: self.visibility.clone(),
            attributes: self.attributes.clone(),
            call_path: self.call_path.clone(),
            span: self.span.clone(),
        })
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
