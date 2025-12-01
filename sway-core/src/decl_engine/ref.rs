//! Represents the use of / syntactic reference to a declaration.
//!
//! ### Is a [DeclRef] effectively the same as a [DeclId]?
//!
//! A [DeclRef] is a smart wrapper around a [DeclId] and canonically represents
//! the use / syntactic reference to a declaration. This does not include the
//! syntactic locations for where declarations are declared though. For example,
//! function declaration `fn my_function() { .. }` would just create a [DeclId],
//! while function application `my_function()` would create a [DeclRef].
//!
//! [DeclRef] contains a [DeclId] field `id`, as well as some additional helpful
//! information. These additional fields include an [Ident] for the declaration
//! `name` and a [Span] for the declaration `decl_span`. Note, `name` and
//! `decl_span` can also be found by using `id` to get the declaration itself
//! from the [DeclEngine]. But the [DeclRef] type allows Sway compiler writers
//! to reduce unnecessary lookups into the [DeclEngine] when only the `name` or
//! `decl_span` is desired.
//!
//! It is recommend to use [DeclId] for cases like function declaration
//! `fn my_function() { .. }`, and to use [DeclRef] for cases like function
//! application `my_function()`.

use crate::{
    decl_engine::*,
    engine_threading::*,
    language::ty::{
        self, TyAbiDecl, TyConstantDecl, TyDeclParsedType, TyEnumDecl, TyFunctionDecl,
        TyImplSelfOrTrait, TyStorageDecl, TyStructDecl, TyTraitDecl, TyTraitFn, TyTraitType,
    },
    semantic_analysis::TypeCheckContext,
    type_system::*,
};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::{Ident, Named, Span, Spanned};

pub type DeclRefFunction = DeclRef<DeclId<TyFunctionDecl>>;
pub type DeclRefTrait = DeclRef<DeclId<TyTraitDecl>>;
pub type DeclRefTraitFn = DeclRef<DeclId<TyTraitFn>>;
pub type DeclRefTraitType = DeclRef<DeclId<TyTraitType>>;
pub type DeclRefImplTrait = DeclRef<DeclId<TyImplSelfOrTrait>>;
pub type DeclRefStruct = DeclRef<DeclId<TyStructDecl>>;
pub type DeclRefStorage = DeclRef<DeclId<TyStorageDecl>>;
pub type DeclRefAbi = DeclRef<DeclId<TyAbiDecl>>;
pub type DeclRefConstant = DeclRef<DeclId<TyConstantDecl>>;
pub type DeclRefEnum = DeclRef<DeclId<TyEnumDecl>>;

pub type DeclRefMixedFunctional = DeclRef<AssociatedItemDeclId>;
pub type DeclRefMixedInterface = DeclRef<InterfaceDeclId>;

/// Represents the use of / syntactic reference to a declaration. A
/// smart-wrapper around a [DeclId], containing additional information about a
/// declaration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeclRef<I> {
    /// The name of the declaration.
    // NOTE: In the case of storage, the name is "storage".
    name: Ident,

    /// The index into the [DeclEngine].
    id: I,

    /// The [Span] of the entire declaration.
    decl_span: Span,
}

impl<I> DeclRef<I> {
    pub(crate) fn new(name: Ident, id: I, decl_span: Span) -> Self {
        DeclRef {
            name,
            id,
            decl_span,
        }
    }

    pub fn name(&self) -> &Ident {
        &self.name
    }

    pub fn id(&self) -> &I {
        &self.id
    }

    pub fn decl_span(&self) -> &Span {
        &self.decl_span
    }
}

impl<T> DeclRef<DeclId<T>> {
    pub(crate) fn replace_id(&mut self, index: DeclId<T>) {
        self.id.replace_id(index);
    }
}

impl<T> DeclRef<DeclId<T>>
where
    DeclEngine: DeclEngineIndex<T> + DeclEngineInsert<T> + DeclEngineGetParsedDeclId<T>,
    T: Named + Spanned + IsConcrete + SubstTypes + Clone + TyDeclParsedType,
{
    pub(crate) fn subst_types_and_insert_new(&self, ctx: &SubstTypesContext) -> Option<Self> {
        let decl_engine = ctx.engines.de();
        if ctx
            .type_subst_map
            .is_some_and(|tsm| tsm.source_ids_contains_concrete_type(ctx.engines))
            || !decl_engine.get(&self.id).is_concrete(ctx.engines)
        {
            let mut decl = (*decl_engine.get(&self.id)).clone();
            if decl.subst(ctx).has_changes() {
                Some(decl_engine.insert(decl, decl_engine.get_parsed_decl_id(&self.id).as_ref()))
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl<T> DeclRef<DeclId<T>>
where
    AssociatedItemDeclId: From<DeclId<T>>,
{
    pub(crate) fn with_parent(
        self,
        decl_engine: &DeclEngine,
        parent: AssociatedItemDeclId,
    ) -> Self {
        let id: DeclId<T> = self.id;
        decl_engine.register_parent(id.into(), parent);
        self
    }
}

impl<T> DeclRef<DeclId<T>>
where
    AssociatedItemDeclId: From<DeclId<T>>,
    DeclEngine: DeclEngineIndex<T> + DeclEngineInsert<T> + DeclEngineGetParsedDeclId<T>,
    T: Named + Spanned + IsConcrete + SubstTypes + Clone + TyDeclParsedType,
{
    pub(crate) fn subst_types_and_insert_new_with_parent(
        &self,
        ctx: &SubstTypesContext,
    ) -> Option<Self> {
        let decl_engine = ctx.engines.de();
        let mut decl = (*decl_engine.get(&self.id)).clone();
        if decl.subst(ctx).has_changes() {
            Some(
                decl_engine
                    .insert(decl, decl_engine.get_parsed_decl_id(&self.id).as_ref())
                    .with_parent(decl_engine, self.id.into()),
            )
        } else {
            None
        }
    }
}

impl<T> EqWithEngines for DeclRef<DeclId<T>>
where
    DeclEngine: DeclEngineIndex<T>,
    T: Named + Spanned + PartialEqWithEngines + EqWithEngines,
{
}

impl<T> PartialEqWithEngines for DeclRef<DeclId<T>>
where
    DeclEngine: DeclEngineIndex<T>,
    T: Named + Spanned + PartialEqWithEngines,
{
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        let decl_engine = ctx.engines().de();
        let DeclRef {
            name: ln,
            id: lid,
            // these fields are not used in comparison because they aren't
            // relevant/a reliable source of obj v. obj distinction
            decl_span: _,
            // temporarily omitted
        } = self;
        let DeclRef {
            name: rn,
            id: rid,
            // these fields are not used in comparison because they aren't
            // relevant/a reliable source of obj v. obj distinction
            decl_span: _,
            // temporarily omitted
        } = other;
        ln == rn && decl_engine.get(lid).eq(&decl_engine.get(rid), ctx)
    }
}

impl<T> HashWithEngines for DeclRef<DeclId<T>>
where
    DeclEngine: DeclEngineIndex<T>,
    T: Named + Spanned + HashWithEngines,
{
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        let decl_engine = engines.de();
        let DeclRef {
            name,
            id,
            // these fields are not hashed because they aren't relevant/a
            // reliable source of obj v. obj distinction
            decl_span: _,
        } = self;
        name.hash(state);
        decl_engine.get(id).hash(state, engines);
    }
}

impl EqWithEngines for DeclRefMixedInterface {}
impl PartialEqWithEngines for DeclRefMixedInterface {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        let decl_engine = ctx.engines().de();
        match (&self.id, &other.id) {
            (InterfaceDeclId::Abi(self_id), InterfaceDeclId::Abi(other_id)) => {
                let left = decl_engine.get(self_id);
                let right = decl_engine.get(other_id);
                self.name == other.name && left.eq(&right, ctx)
            }
            (InterfaceDeclId::Trait(self_id), InterfaceDeclId::Trait(other_id)) => {
                let left = decl_engine.get(self_id);
                let right = decl_engine.get(other_id);
                self.name == other.name && left.eq(&right, ctx)
            }
            _ => false,
        }
    }
}

impl HashWithEngines for DeclRefMixedInterface {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        match self.id {
            InterfaceDeclId::Abi(id) => {
                state.write_u8(0);
                let decl_engine = engines.de();
                let decl = decl_engine.get(&id);
                decl.hash(state, engines);
            }
            InterfaceDeclId::Trait(id) => {
                state.write_u8(1);
                let decl_engine = engines.de();
                let decl = decl_engine.get(&id);
                decl.hash(state, engines);
            }
        }
    }
}

impl<I> Spanned for DeclRef<I> {
    fn span(&self) -> Span {
        self.decl_span.clone()
    }
}

// impl<T> SubstTypes for DeclRef<DeclId<T>>
// where
//     DeclEngine: DeclEngineIndex<T>,
//     T: Named + Spanned + SubstTypes + Clone,
// {
//     fn subst_inner(&mut self, ctx: &SubstTypesContext) -> HasChanges {
//         let decl_engine = ctx.engines.de();
//         let mut decl = (*decl_engine.get(&self.id)).clone();
//         if decl.subst(ctx).has_changes() {
//             decl_engine.replace(self.id, decl);
//             HasChanges::Yes
//         } else {
//             HasChanges::No
//         }
//     }
// }

impl ReplaceDecls for DeclRefFunction {
    fn replace_decls_inner(
        &mut self,
        decl_mapping: &DeclMapping,
        handler: &Handler,
        ctx: &mut TypeCheckContext,
    ) -> Result<bool, ErrorEmitted> {
        let engines = ctx.engines();
        let decl_engine = engines.de();

        let func = decl_engine.get(self);

        if let Some(new_decl_ref) = decl_mapping.find_match(
            handler,
            ctx.engines(),
            self.id.into(),
            func.implementing_for,
            ctx.self_type(),
        )? {
            return Ok(
                if let AssociatedItemDeclId::Function(new_decl_ref) = new_decl_ref {
                    self.id = new_decl_ref;
                    true
                } else {
                    false
                },
            );
        }
        let all_parents = decl_engine.find_all_parents(engines, &self.id);
        for parent in all_parents.iter() {
            if let Some(new_decl_ref) = decl_mapping.find_match(
                handler,
                ctx.engines(),
                parent.clone(),
                func.implementing_for,
                ctx.self_type(),
            )? {
                return Ok(
                    if let AssociatedItemDeclId::Function(new_decl_ref) = new_decl_ref {
                        self.id = new_decl_ref;
                        true
                    } else {
                        false
                    },
                );
            }
        }
        Ok(false)
    }
}

impl ReplaceFunctionImplementingType for DeclRefFunction {
    fn replace_implementing_type(&mut self, engines: &Engines, implementing_type: ty::TyDecl) {
        let decl_engine = engines.de();
        let mut decl = (*decl_engine.get(&self.id)).clone();
        decl.set_implementing_type(implementing_type);
        decl_engine.replace(self.id, decl);
    }
}
