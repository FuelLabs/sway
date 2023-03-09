use sway_error::error::CompileError;
use sway_types::Spanned;

use crate::{
    error::*,
    language::{ty, CallPath, Visibility},
    CompileResult, Engines, Ident,
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
        mod_path: &Path,
        call_path: &CallPath,
    ) -> CompileResult<&ty::TyDeclaration> {
        let symbol_path: Vec<_> = mod_path
            .iter()
            .chain(&call_path.prefixes)
            .cloned()
            .collect();
        self.resolve_symbol(&symbol_path, &call_path.suffix)
    }

    /// Resolve a symbol that is potentially prefixed with some path, e.g. `foo::bar::symbol`.
    ///
    /// This is short-hand for concatenating the `mod_path` with the `call_path`'s prefixes and
    /// then calling `resolve_symbol` with the resulting path and call_path's suffix.
    ///
    /// When `call_path` contains prefixes and the resolved declaration visibility is not public
    /// an error is thrown.
    pub(crate) fn resolve_call_path_with_visibility_check(
        &self,
        engines: Engines<'_>,
        mod_path: &Path,
        call_path: &CallPath,
    ) -> CompileResult<&ty::TyDeclaration> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let result = self.resolve_call_path(mod_path, call_path);

        // In case there are no prefixes we don't need to check visibility
        if call_path.prefixes.is_empty() {
            return result;
        }

        if let CompileResult {
            value: Some(decl), ..
        } = result
        {
            let visibility = check!(
                decl.visibility(engines.de()),
                return err(warnings, errors),
                warnings,
                errors
            );
            if visibility != Visibility::Public {
                errors.push(CompileError::ImportPrivateSymbol {
                    name: call_path.suffix.clone(),
                    span: call_path.suffix.span(),
                });
                // Returns ok with error, this allows functions which call this to
                // also access the returned TyDeclaration and throw more suitable errors.
                return ok(decl, warnings, errors);
            }
        }

        result
    }

    /// Given a path to a module and the identifier of a symbol within that module, resolve its
    /// declaration.
    ///
    /// If the symbol is within the given module's namespace via import, we recursively traverse
    /// imports until we find the original declaration.
    pub(crate) fn resolve_symbol(
        &self,
        mod_path: &Path,
        symbol: &Ident,
    ) -> CompileResult<&ty::TyDeclaration> {
        self.check_submodule(mod_path).flat_map(|module| {
            let true_symbol = self[mod_path]
                .use_aliases
                .get(symbol.as_str())
                .unwrap_or(symbol);
            match module.use_synonyms.get(symbol) {
                Some((src_path, _, _)) if mod_path != src_path => {
                    self.resolve_symbol(src_path, true_symbol)
                }
                _ => CompileResult::from(module.check_symbol(true_symbol)),
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
