use crate::{
    error::*, semantic_analysis::*, type_system::*, CallPath, CompileResult, Ident, TypeInfo,
    TypedDeclaration, TypedFunctionDeclaration,
};

use super::{module::Module, namespace::Namespace, Path};

use sway_types::Spanned;

use std::collections::VecDeque;

/// The root module, from which all other modules can be accessed.
///
/// This is equivalent to the "crate root" of a Rust crate.
///
/// We use a custom type for the `Root` in order to ensure that methods that only work with
/// canonical paths, or that use canonical paths internally, are *only* called from the root. This
/// normally includes methods that first lookup some canonical path via `use_synonyms` before using
/// that canonical path to look up the symbol declaration.
#[derive(Clone, Debug, PartialEq)]
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
    ) -> CompileResult<&TypedDeclaration> {
        let symbol_path: Vec<_> = mod_path
            .iter()
            .chain(&call_path.prefixes)
            .cloned()
            .collect();
        self.resolve_symbol(&symbol_path, &call_path.suffix)
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
    ) -> CompileResult<&TypedDeclaration> {
        self.check_submodule(mod_path).flat_map(|module| {
            let true_symbol = self[mod_path]
                .use_aliases
                .get(symbol.as_str())
                .unwrap_or(symbol);
            match module.use_synonyms.get(symbol) {
                Some((src_path, _)) if mod_path != src_path => {
                    self.resolve_symbol(src_path, true_symbol)
                }
                _ => CompileResult::from(module.check_symbol(true_symbol)),
            }
        })
    }

    /// Given a method and a type (plus a `self_type` to potentially resolve it), find that method
    /// in the namespace. Requires `args_buf` because of some special casing for the standard
    /// library where we pull the type from the arguments buffer.
    ///
    /// This function will generate a missing method error if the method is not found.
    ///
    /// This method should only be called on the root namespace. `mod_path` is the current module,
    /// `method_path` is assumed to be absolute.
    pub(crate) fn find_method_for_type(
        &mut self,
        mod_path: &Path,
        type_id: TypeId,
        method_prefix: &Path,
        method_name: &Ident,
        self_type: TypeId,
        args_buf: &VecDeque<TypedExpression>,
    ) -> CompileResult<TypedFunctionDeclaration> {
        let mut warnings = vec![];
        let mut errors = vec![];

        // grab the local module
        let local_module = check!(
            self.check_submodule(mod_path),
            return err(warnings, errors),
            warnings,
            errors
        );

        // grab the local methods from the local module
        let local_methods = local_module.get_methods_for_type(type_id);

        type_id.replace_self_type(self_type);

        // resolve the type
        append!(
            resolve_type(
                type_id,
                &method_name.span(),
                EnforceTypeArguments::No,
                None,
                self,
                method_prefix,
            ),
            warnings,
            errors
        );

        // grab the module where the type itself is declared
        let type_module = check!(
            self.check_submodule(method_prefix),
            return err(warnings, errors),
            warnings,
            errors
        );

        // grab the methods from where the type is declared
        let mut type_methods = type_module.get_methods_for_type(type_id);

        let mut methods = local_methods;
        methods.append(&mut type_methods);

        match methods
            .into_iter()
            .find(|TypedFunctionDeclaration { name, .. }| name == method_name)
        {
            Some(o) => ok(o, warnings, errors),
            None => {
                if args_buf.get(0).map(|x| look_up_type_id(x.return_type))
                    != Some(TypeInfo::ErrorRecovery)
                {
                    errors.push(CompileError::MethodNotFound {
                        method_name: method_name.clone(),
                        type_name: type_id.to_string(),
                    });
                }
                err(warnings, errors)
            }
        }
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
