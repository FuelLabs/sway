use crate::{engine_threading::Engines, type_system::priv_prelude::*, HasChanges};
use sway_error::handler::Handler;
use sway_types::Ident;

pub struct SubstTypesContext<'a> {
    pub handler: &'a Handler,
    pub engines: &'a Engines,
    pub type_subst_map: Option<&'a TypeSubstMap>,
    pub subst_function_body: bool,
}

impl<'a> SubstTypesContext<'a> {
    pub fn new(
        handler: &'a Handler,
        engines: &'a Engines,
        type_subst_map: &'a TypeSubstMap,
        subst_function_body: bool,
    ) -> SubstTypesContext<'a> {
        SubstTypesContext {
            handler,
            engines,
            type_subst_map: Some(type_subst_map),
            subst_function_body,
        }
    }

    pub fn dummy(handler: &'a Handler, engines: &'a Engines) -> SubstTypesContext<'a> {
        SubstTypesContext {
            handler,
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
