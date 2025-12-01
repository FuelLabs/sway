use std::borrow::Cow;

use crate::{
    engine_threading::Engines,
    semantic_analysis::Visitor,
    type_system::priv_prelude::*,
};
use sway_types::Ident;

#[derive(Default)]
pub enum HasChanges {
    Yes,
    #[default]
    No,
}

impl HasChanges {
    pub fn has_changes(&self) -> bool {
        matches!(self, HasChanges::Yes)
    }
}

impl std::ops::BitOr for HasChanges {
    type Output = HasChanges;

    fn bitor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (HasChanges::No, HasChanges::No) => HasChanges::No,
            _ => HasChanges::Yes,
        }
    }
}

pub struct SubstTypesContext<'eng, 'tsm> {
    pub engines: &'eng Engines,
    pub type_subst_map: Option<&'tsm TypeSubstMap>,
    pub subst_function_body: bool,
}

impl<'eng, 'tsm> SubstTypesContext<'eng, 'tsm> {
    pub fn new(
        engines: &'eng Engines,
        type_subst_map: &'tsm TypeSubstMap,
        subst_function_body: bool,
    ) -> SubstTypesContext<'eng, 'tsm> {
        SubstTypesContext {
            engines,
            type_subst_map: Some(type_subst_map),
            subst_function_body,
        }
    }

    pub fn dummy(engines: &'eng Engines) -> SubstTypesContext<'eng, 'tsm> {
        SubstTypesContext {
            engines,
            type_subst_map: None,
            subst_function_body: false,
        }
    }

    pub fn get_renamed_const_generic(&self, name: &Ident) -> Option<&sway_types::BaseIdent> {
        self.type_subst_map
            .as_ref()
            .and_then(|map| map.const_generics_renaming.get(name))
    }
}

pub struct ReplaceTypesVisitor<'ctx, 'eng, 'tsm> {
    pub ctx: &'ctx SubstTypesContext<'eng, 'tsm>,
}

impl<'ctx, 'eng, 'tsm> Visitor for ReplaceTypesVisitor<'ctx, 'eng, 'tsm> {
    const VISIT_GENERIC_TYPE_ARGUMENT_INITIAL_TYPE_ID: bool = false;

    fn visit_type_id<'a>(&mut self, type_id: &'a TypeId) -> Cow<'a, TypeId> {
        let type_engine = self.ctx.engines.te();
        if let Some(matching_id) = self
            .ctx
            .type_subst_map
            .and_then(|tsm| tsm.find_match(*type_id, self.ctx.engines))
        {
            // TODO cheaper to never include ErrorRecovery in TypeSubstMap
            if !matches!(&*type_engine.get(matching_id), TypeInfo::ErrorRecovery(_)) {
                return Cow::Owned(matching_id);
            }
        }

        Cow::Borrowed(type_id)
    }
}

pub trait SubstTypes {
    fn subst_inner(&mut self, ctx: &SubstTypesContext) -> HasChanges;

    fn subst(&mut self, ctx: &SubstTypesContext) -> HasChanges {
        if ctx.type_subst_map.is_some_and(|tsm| tsm.is_empty()) {
            HasChanges::No
        } else {
            self.subst_inner(ctx)
        }
    }
}

impl<T: SubstTypes + Clone> SubstTypes for std::sync::Arc<T> {
    fn subst_inner(&mut self, ctx: &SubstTypesContext) -> HasChanges {
        if let Some(item) = std::sync::Arc::get_mut(self) {
            item.subst_inner(ctx)
        } else {
            let mut item = self.as_ref().clone();
            let r = item.subst_inner(ctx);
            *self = std::sync::Arc::new(item);
            r
        }
    }
}

impl<A, B: SubstTypes> SubstTypes for (A, B) {
    fn subst_inner(&mut self, ctx: &SubstTypesContext) -> HasChanges {
        self.1.subst(ctx)
    }
}

impl<T: SubstTypes> SubstTypes for Box<T> {
    fn subst_inner(&mut self, ctx: &SubstTypesContext) -> HasChanges {
        self.as_mut().subst(ctx)
    }
}

impl<T: SubstTypes> SubstTypes for Option<T> {
    fn subst_inner(&mut self, ctx: &SubstTypesContext) -> HasChanges {
        self.as_mut().map(|x| x.subst(ctx)).unwrap_or_default()
    }
}

impl<T: SubstTypes> SubstTypes for Vec<T> {
    fn subst_inner(&mut self, ctx: &SubstTypesContext) -> HasChanges {
        self.iter_mut()
            .fold(HasChanges::No, |has_change, x| x.subst(ctx) | has_change)
    }
}

#[macro_export]
macro_rules! has_changes {
    ($($stmt:expr);* ;) => {{
        let mut has_changes = $crate::type_system::HasChanges::No;
        $(
            has_changes = $stmt | has_changes;
        )*
        has_changes
    }};
}
