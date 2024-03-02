use crate::{
    decl_engine::DeclRef,
    engine_threading::*,
    language::{
	ty::{self, TyDecl},
    },
    namespace::Path,
};
use super::{
    lexical_scope::GlobImport,
    module::Module,
    namespace::Namespace,
    trait_map::TraitMap,
    Ident, 
};
use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler}
};
use sway_types::Spanned;
use sway_utils::iter_prefixes;

/// The root module, from which all other modules can be accessed.
///
/// This is equivalent to the "crate root" of a Rust crate.
///
/// We use a custom type for the `Root` in order to ensure that methods that only work with
/// canonical paths, or that use canonical paths internally, are *only* called from the root. This
/// normally includes methods that first lookup some canonical path via `use_synonyms` before using
/// that canonical path to look up the symbol declaration.
#[derive(Clone, Debug)]
pub struct Root {
    pub(crate) module: Module,
}

impl Root {

    /// Given a path to a `src` module, create synonyms to every symbol in that module to the given
    /// `dst` module.
    ///
    /// This is used when an import path contains an asterisk.
    ///
    /// Paths are assumed to be absolute.
    pub(crate) fn star_import(
        &mut self,
        handler: &Handler,
        engines: &Engines,
        src: &Path,
        dst: &Path,
    ) -> Result<(), ErrorEmitted> {
        self.check_module_privacy(handler, src)?;

        let decl_engine = engines.de();

        let src_mod = self.module.check_submodule(handler, src)?;

        let implemented_traits = src_mod.current_items().implemented_traits.clone();
        let mut symbols_and_decls = vec![];
        for (symbol, decl) in src_mod.current_items().symbols.iter() {
            if is_ancestor(src, dst) || decl.visibility(decl_engine).is_public() {
                symbols_and_decls.push((symbol.clone(), decl.clone()));
            }
        }

        let dst_mod = &mut self.module[dst];
        dst_mod
            .current_items_mut()
            .implemented_traits
            .extend(implemented_traits, engines);  // TODO: No difference made between imported and declared items
        for symbol_and_decl in symbols_and_decls {
            dst_mod.current_items_mut().use_synonyms.insert( // TODO: No difference made between imported and declared items
                symbol_and_decl.0,
                (
                    src.to_vec(),
                    GlobImport::Yes,
                    symbol_and_decl.1,
                    true,
                ),
            );
        }

        Ok(())
    }

    /// Pull a single item from a `src` module and import it into the `dst` module.
    ///
    /// The item we want to import is basically the last item in path because this is a `self`
    /// import.
    pub(crate) fn self_import(
        &mut self,
        handler: &Handler,
        engines: &Engines,
        src: &Path,
        dst: &Path,
        alias: Option<Ident>,
    ) -> Result<(), ErrorEmitted> {
        let (last_item, src) = src.split_last().expect("guaranteed by grammar");
        self.item_import(
            handler,
            engines,
            src,
            last_item,
            dst,
            alias,
        )
    }

    /// Pull a single `item` from the given `src` module and import it into the `dst` module.
    ///
    /// Paths are assumed to be absolute.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn item_import(
        &mut self,
        handler: &Handler,
        engines: &Engines,
        src: &Path,
        item: &Ident,
        dst: &Path,
        alias: Option<Ident>,
    ) -> Result<(), ErrorEmitted> {
        self.check_module_privacy(handler, src)?;

        let decl_engine = engines.de();

        let src_mod = self.module.check_submodule(handler, src)?;
        let mut impls_to_insert = TraitMap::default();
        match src_mod.current_items().symbols.get(item).cloned() {
            Some(decl) => {
                if !decl.visibility(decl_engine).is_public() && !is_ancestor(src, dst) {
                    handler.emit_err(CompileError::ImportPrivateSymbol {
                        name: item.clone(),
                        span: item.span(),
                    });
                }

                //  if this is an enum or struct or function, import its implementations
                if let Ok(type_id) = decl.return_type(&Handler::default(), engines) {
                    impls_to_insert.extend(
                        src_mod
                            .current_items()
                            .implemented_traits
                            .filter_by_type_item_import(type_id, engines),
                        engines,
                    );
                }
                // if this is a trait, import its implementations
                let decl_span = decl.span();
                if let TyDecl::TraitDecl(_) = &decl {
                    // TODO: we only import local impls from the source namespace
                    // this is okay for now but we'll need to device some mechanism to collect all available trait impls
                    impls_to_insert.extend(
                        src_mod
                            .current_items()
                            .implemented_traits
                            .filter_by_trait_decl_span(decl_span),
                        engines,
                    );
                }
                // no matter what, import it this way though.
                let dst_mod = &mut self.module[dst];
                let add_synonym = |name| {
                    if let Some((_, GlobImport::No, _, _)) =
                        dst_mod.current_items().use_synonyms.get(name)
                    {
                        handler.emit_err(CompileError::ShadowsOtherSymbol { name: name.into() });
                    }
                    dst_mod.current_items_mut().use_synonyms.insert(   // TODO: No difference made between imported and declared items
                        name.clone(),
                        (src.to_vec(), GlobImport::No, decl, true),
                    );
                };
                match alias {
                    Some(alias) => {
                        add_synonym(&alias);
                        dst_mod
                            .current_items_mut()
                            .use_aliases
                            .insert(alias.as_str().to_string(), item.clone());   // TODO: No difference made between imported and declared items
                    }
                    None => add_synonym(item),
                };
            }
            None => {
                return Err(handler.emit_err(CompileError::SymbolNotFound {
                    name: item.clone(),
                    span: item.span(),
                }));
            }
        };

        let dst_mod = &mut self.module[dst];
        dst_mod
            .current_items_mut()
            .implemented_traits
            .extend(impls_to_insert, engines);   // TODO: No difference made between imported and declared items

        Ok(())
    }

    /// Pull a single variant `variant` from the enum `enum_name` from the given `src` module and import it into the `dst` module.
    ///
    /// Paths are assumed to be absolute.
    #[allow(clippy::too_many_arguments)] // TODO: remove lint bypass once private modules are no longer experimental
    pub(crate) fn variant_import(
        &mut self,
        handler: &Handler,
        engines: &Engines,
        src: &Path,
        enum_name: &Ident,
        variant_name: &Ident,
        dst: &Path,
        alias: Option<Ident>,
    ) -> Result<(), ErrorEmitted> {
        self.check_module_privacy(handler, src)?;

        let decl_engine = engines.de();

        let src_mod = self.module.check_submodule(handler, src)?;
        match src_mod.current_items().symbols.get(enum_name).cloned() {
            Some(decl) => {
                if !decl.visibility(decl_engine).is_public() && !is_ancestor(src, dst) {
                    handler.emit_err(CompileError::ImportPrivateSymbol {
                        name: enum_name.clone(),
                        span: enum_name.span(),
                    });
                }

                if let TyDecl::EnumDecl(ty::EnumDecl {
                    decl_id,
                    subst_list: _,
                    ..
                }) = decl
                {
                    let enum_decl = decl_engine.get_enum(&decl_id);
                    let enum_ref = DeclRef::new(
                        enum_decl.call_path.suffix.clone(),
                        decl_id,
                        enum_decl.span(),
                    );

                    if let Some(variant_decl) =
                        enum_decl.variants.iter().find(|v| v.name == *variant_name)
                    {
                        // import it this way.
                        let dst_mod = &mut self.module[dst];
                        let mut add_synonym = |name| {
                            if let Some((_, GlobImport::No, _, _)) =
                                dst_mod.current_items().use_synonyms.get(name)
                            {
                                handler.emit_err(CompileError::ShadowsOtherSymbol {
                                    name: name.into(),
                                });
                            }
                            dst_mod.current_items_mut().use_synonyms.insert(   // TODO: No difference made between imported and declared items
                                name.clone(),
                                (
                                    src.to_vec(),
                                    GlobImport::No,
                                    TyDecl::EnumVariantDecl(ty::EnumVariantDecl {
                                        enum_ref: enum_ref.clone(),
                                        variant_name: variant_name.clone(),
                                        variant_decl_span: variant_decl.span.clone(),
                                    }),
                                    true,
                                ),
                            );
                        };
                        match alias {
                            Some(alias) => {
                                add_synonym(&alias);
                                dst_mod
                                    .current_items_mut()
                                    .use_aliases
                                    .insert(alias.as_str().to_string(), variant_name.clone());    // TODO: No difference made between imported and declared items
                            }
                            None => add_synonym(variant_name),
                        };
                    } else {
                        return Err(handler.emit_err(CompileError::SymbolNotFound {
                            name: variant_name.clone(),
                            span: variant_name.span(),
                        }));
                    }
                } else {
                    return Err(handler.emit_err(CompileError::Internal(
                        "Attempting to import variants of something that isn't an enum",
                        enum_name.span(),
                    )));
                }
            }
            None => {
                return Err(handler.emit_err(CompileError::SymbolNotFound {
                    name: enum_name.clone(),
                    span: enum_name.span(),
                }));
            }
        };

        Ok(())
    }

    /// Pull all variants from the enum `enum_name` from the given `src` module and import them all into the `dst` module.
    ///
    /// Paths are assumed to be absolute.
    pub(crate) fn variant_star_import(
        &mut self,
        handler: &Handler,
        engines: &Engines,
        src: &Path,
        dst: &Path,
        enum_name: &Ident,
    ) -> Result<(), ErrorEmitted> {
        self.check_module_privacy(handler, src)?;

        let decl_engine = engines.de();

        let src_mod = self.module.check_submodule(handler, src)?;
        match src_mod.current_items().symbols.get(enum_name).cloned() {
            Some(decl) => {
                if !decl.visibility(decl_engine).is_public() && !is_ancestor(src, dst) {
                    handler.emit_err(CompileError::ImportPrivateSymbol {
                        name: enum_name.clone(),
                        span: enum_name.span(),
                    });
                }

                if let TyDecl::EnumDecl(ty::EnumDecl {
                    decl_id,
                    subst_list: _,
                    ..
                }) = decl
                {
                    let enum_decl = decl_engine.get_enum(&decl_id);
                    let enum_ref = DeclRef::new(
                        enum_decl.call_path.suffix.clone(),
                        decl_id,
                        enum_decl.span(),
                    );

                    for variant_decl in enum_decl.variants.iter() {
                        let variant_name = &variant_decl.name;

                        // import it this way.
                        let dst_mod = &mut self.module[dst];
                        dst_mod.current_items_mut().use_synonyms.insert(   // TODO: No difference made between imported and declared items
                            variant_name.clone(),
                            (
                                src.to_vec(),
                                GlobImport::Yes,
                                TyDecl::EnumVariantDecl(ty::EnumVariantDecl {
                                    enum_ref: enum_ref.clone(),
                                    variant_name: variant_name.clone(),
                                    variant_decl_span: variant_decl.span.clone(),
                                }),
				true
                            ),
                        );
                    }
                } else {
                    return Err(handler.emit_err(CompileError::Internal(
                        "Attempting to import variants of something that isn't an enum",
                        enum_name.span(),
                    )));
                }
            }
            None => {
                return Err(handler.emit_err(CompileError::SymbolNotFound {
                    name: enum_name.clone(),
                    span: enum_name.span(),
                }));
            }
        };

        Ok(())
    }

    fn check_module_privacy(&self, handler: &Handler, src: &Path) -> Result<(), ErrorEmitted> {
        let dst = &self.module.mod_path;
        // you are always allowed to access your ancestor's symbols
        if !is_ancestor(src, dst) {
            // we don't check the first prefix because direct children are always accessible
            for prefix in iter_prefixes(src).skip(1) {
                let module = self.module.check_submodule(handler, prefix)?;
                if module.visibility.is_private() {
                    let prefix_last = prefix[prefix.len() - 1].clone();
                    handler.emit_err(CompileError::ImportPrivateModule {
                        span: prefix_last.span(),
                        name: prefix_last,
                    });
                }
            }
        }
        Ok(())
    }
}

impl From<Module> for Root {
    fn from(module: Module) -> Self {
        Root { module }
    }
}

impl From<Namespace> for Root {
    fn from(namespace: Namespace) -> Self {
        namespace.root
    }
}

fn is_ancestor(src: &Path, dst: &Path) -> bool {
    dst.len() >= src.len() && src.iter().zip(dst).all(|(src, dst)| src == dst)
}
