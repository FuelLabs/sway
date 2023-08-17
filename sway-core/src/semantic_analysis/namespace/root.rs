use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::Spanned;
use sway_utils::iter_prefixes;

use crate::{
    language::{ty, CallPath},
    Engines, Ident,
};

use super::{module::Module, namespace::Namespace, Path};

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
    /// Resolve a symbol that is potentially prefixed with some path, e.g. `foo::bar::symbol`.
    ///
    /// This is short-hand for concatenating the `mod_path` with the `call_path`'s prefixes and
    /// then calling `resolve_symbol` with the resulting path and call_path's suffix.
    pub(crate) fn resolve_call_path(
        &self,
        handler: &Handler,
        mod_path: &Path,
        call_path: &CallPath,
    ) -> Result<&ty::TyDecl, ErrorEmitted> {
        let symbol_path: Vec<_> = mod_path
            .iter()
            .chain(&call_path.prefixes)
            .cloned()
            .collect();
        self.resolve_symbol(handler, &symbol_path, &call_path.suffix)
    }

    /// Resolve a symbol that is potentially prefixed with some path, e.g. `foo::bar::symbol`.
    ///
    /// This will concatenate the `mod_path` with the `call_path`'s prefixes and
    /// then calling `resolve_symbol` with the resulting path and call_path's suffix.
    ///
    /// The `mod_path` is significant here as we assume the resolution is done within the
    /// context of the module pointed to by `mod_path` and will only check the call path prefixes
    /// and the symbol's own visibility
    pub(crate) fn resolve_call_path_with_visibility_check(
        &self,
        handler: &Handler,
        engines: &Engines,
        mod_path: &Path,
        call_path: &CallPath,
    ) -> Result<&ty::TyDecl, ErrorEmitted> {
        let decl = self.resolve_call_path(handler, mod_path, call_path)?;

        // In case there are no prefixes we don't need to check visibility
        if call_path.prefixes.is_empty() {
            return Ok(decl);
        }

        // check the visibility of the call path elements
        // we don't check the first prefix because direct children are always accessible
        for prefix in iter_prefixes(&call_path.prefixes).skip(1) {
            let module = self.check_submodule(handler, prefix)?;
            if module.visibility.is_private() {
                let prefix_last = prefix[prefix.len() - 1].clone();
                handler.emit_err(CompileError::ImportPrivateModule {
                    span: prefix_last.span(),
                    name: prefix_last,
                });
            }
        }

        // check the visibility of the symbol itself
        if !decl.visibility(engines.de()).is_public() {
            handler.emit_err(CompileError::ImportPrivateSymbol {
                name: call_path.suffix.clone(),
                span: call_path.suffix.span(),
            });
        }

        Ok(decl)
    }

    /// Given a path to a module and the identifier of a symbol within that module, resolve its
    /// declaration.
    ///
    /// If the symbol is within the given module's namespace via import, we recursively traverse
    /// imports until we find the original declaration.
    pub(crate) fn resolve_symbol(
        &self,
        handler: &Handler,
        mod_path: &Path,
        symbol: &Ident,
    ) -> Result<&ty::TyDecl, ErrorEmitted> {
        self.check_submodule(handler, mod_path).and_then(|module| {
            let true_symbol = self[mod_path]
                .use_aliases
                .get(symbol.as_str())
                .unwrap_or(symbol);
            match module.use_synonyms.get(symbol) {
                Some((_, _, decl @ ty::TyDecl::EnumVariantDecl { .. }, _)) => Ok(decl),
                Some((src_path, _, _, _)) if mod_path != src_path => {
                    // TODO: check that the symbol import is public?
                    self.resolve_symbol(handler, src_path, true_symbol)
                }
                _ => module
                    .check_symbol(true_symbol)
                    .map_err(|e| handler.emit_err(e)),
            }
        })
    }
}

impl std::ops::Deref for Root {
    type Target = Module;
    fn deref(&self) -> &Self::Target {
        &self.module
    }
}

impl std::ops::DerefMut for Root {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.module
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
