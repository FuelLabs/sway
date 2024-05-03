use crate::{
    language::{parsed::Declaration, Visibility},
    semantic_analysis::Namespace,
    Engines,
};
use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::{span::Span, Ident};

#[derive(Clone)]
/// Contextual state tracked and accumulated throughout symbol collecting.
pub struct SymbolCollectionContext {
    /// The namespace context accumulated throughout symbol collecting.
    pub(crate) namespace: Namespace,
}

impl SymbolCollectionContext {
    /// Initialize a context at the top-level of a module with its namespace.
    pub fn new(namespace: Namespace) -> Self {
        Self { namespace }
    }

    /// Scope the `CollectionContext` with a new lexical scope.
    pub fn scoped<T>(
        self,
        engines: &Engines,
        with_scoped_ctx: impl FnOnce(SymbolCollectionContext) -> Result<T, ErrorEmitted>,
    ) -> Result<T, ErrorEmitted> {
        self.namespace
            .module(engines)
            .write()
            .unwrap()
            .write(engines, |m| m.push_new_lexical_scope());
        let ret = with_scoped_ctx(self.clone());
        self.namespace
            .module(engines)
            .write()
            .unwrap()
            .write(engines, |m| m.pop_lexical_scope());
        ret
    }

    /// Enter the submodule with the given name and produce a collection context ready for
    /// collecting its content.
    ///
    /// Returns the result of the given `with_submod_ctx` function.
    pub fn enter_submodule<T>(
        &mut self,
        engines: &Engines,
        mod_name: Ident,
        visibility: Visibility,
        module_span: Span,
        with_submod_ctx: impl FnOnce(&mut SymbolCollectionContext) -> T,
    ) -> T {
        self.namespace
            .push_new_submodule(engines, mod_name, visibility, module_span);
        //let Self { namespace, .. } = self;
        //let mut submod_ns = namespace.enter_submodule(mod_name, visibility, module_span);
        let ret = with_submod_ctx(self);
        self.namespace.pop_submodule();
        ret
    }

    /// Short-hand for calling [Items::insert_parsed_symbol].
    pub(crate) fn insert_parsed_symbol(
        &mut self,
        _handler: &Handler,
        engines: &Engines,
        name: Ident,
        item: Declaration,
    ) -> Result<(), ErrorEmitted> {
        self.namespace
            .module(engines)
            .write()
            .unwrap()
            .write(engines, |m| {
                m.current_items_mut()
                    .insert_parsed_symbol(name.clone(), item.clone())
            })
    }
}
