use crate::{
    error::*,
    parse_tree::Visibility,
    semantic_analysis::{
        ast_node::{
            TypedExpression, TypedStorageDeclaration, TypedStructField, TypedVariableDeclaration,
        },
        declaration::{TypedStorageField, VariableMutability},
        TypeCheckedStorageAccess,
    },
    type_engine::*,
    CallPath, CompileResult, Ident, TypeArgument, TypeInfo, TypeParameter, TypedDeclaration,
    TypedFunctionDeclaration,
};

use super::{items::Items, module::Module, namespace::Namespace, ModuleName, Path};

use sway_types::span::Span;

use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

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
                Some(src_path) if mod_path != src_path => {
                    self.resolve_symbol(src_path, true_symbol)
                }
                _ => module.check_symbol(true_symbol),
            }
        })
    }

    /// This function either returns a struct (i.e. custom type), `None`, denoting the type that is
    /// being looked for is actually a generic, not-yet-resolved type.
    ///
    /// If a self type is given and anything on this ref chain refers to self, update the chain.
    pub(crate) fn resolve_type_with_self(
        &mut self,
        mod_path: &Path,
        ty: TypeInfo,
        self_type: TypeId,
        span: Span,
        enforce_type_args: bool,
    ) -> CompileResult<TypeId> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let type_id = match ty {
            TypeInfo::Custom {
                ref name,
                type_arguments,
            } => {
                let mut new_type_arguments = vec![];
                for type_argument in type_arguments.into_iter() {
                    let new_type_id = check!(
                        self.resolve_type_with_self(
                            mod_path,
                            look_up_type_id(type_argument.type_id),
                            self_type,
                            type_argument.span.clone(),
                            enforce_type_args
                        ),
                        insert_type(TypeInfo::ErrorRecovery),
                        warnings,
                        errors
                    );
                    let type_argument = TypeArgument {
                        type_id: new_type_id,
                        span: type_argument.span,
                    };
                    new_type_arguments.push(type_argument);
                }
                match self
                    .resolve_symbol(mod_path, name)
                    .ok(&mut warnings, &mut errors)
                    .cloned()
                {
                    Some(TypedDeclaration::StructDeclaration(decl)) => {
                        if enforce_type_args
                            && new_type_arguments.is_empty()
                            && !decl.type_parameters.is_empty()
                        {
                            errors.push(CompileError::NeedsTypeArguments {
                                name: name.clone(),
                                span: name.span().clone(),
                            });
                            return err(warnings, errors);
                        }
                        if !decl.type_parameters.is_empty() {
                            let new_decl = check!(
                                decl.monomorphize(
                                    &mut self[mod_path],
                                    &new_type_arguments,
                                    Some(self_type)
                                ),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );
                            new_decl.type_id()
                        } else {
                            decl.type_id()
                        }
                    }
                    Some(TypedDeclaration::EnumDeclaration(decl)) => {
                        if enforce_type_args
                            && new_type_arguments.is_empty()
                            && !decl.type_parameters.is_empty()
                        {
                            errors.push(CompileError::NeedsTypeArguments {
                                name: name.clone(),
                                span: name.span().clone(),
                            });
                            return err(warnings, errors);
                        }
                        if !decl.type_parameters.is_empty() {
                            let new_decl = check!(
                                decl.monomorphize_with_type_arguments(
                                    &mut self[mod_path],
                                    &new_type_arguments,
                                    Some(self_type)
                                ),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );
                            new_decl.type_id()
                        } else {
                            decl.type_id()
                        }
                    }
                    Some(TypedDeclaration::GenericTypeForFunctionScope { name, .. }) => {
                        insert_type(TypeInfo::UnknownGeneric { name })
                    }
                    _ => {
                        errors.push(CompileError::UnknownType { span });
                        return err(warnings, errors);
                    }
                }
            }
            TypeInfo::Array(type_id, size) => {
                let elem_type_id = check!(
                    self.resolve_type_with_self(
                        mod_path,
                        look_up_type_id(type_id),
                        self_type,
                        span,
                        enforce_type_args
                    ),
                    insert_type(TypeInfo::ErrorRecovery),
                    warnings,
                    errors
                );
                insert_type(TypeInfo::Array(elem_type_id, size))
            }
            TypeInfo::SelfType => self_type,
            TypeInfo::Ref(id) => id,
            o => insert_type(o),
        };
        ok(type_id, warnings, errors)
    }

    pub(crate) fn resolve_type_without_self(
        &mut self,
        mod_path: &Path,
        ty: &TypeInfo,
    ) -> CompileResult<TypeId> {
        let ty = ty.clone();
        let mut warnings = vec![];
        let mut errors = vec![];
        let type_id = match ty {
            TypeInfo::Custom {
                name,
                type_arguments,
            } => match self
                .resolve_symbol(mod_path, &name)
                .ok(&mut warnings, &mut errors)
                .cloned()
            {
                Some(TypedDeclaration::StructDeclaration(decl)) => {
                    let mut new_type_arguments = vec![];
                    for type_argument in type_arguments.into_iter() {
                        let new_type_id = check!(
                            self.resolve_type_without_self(
                                mod_path,
                                &look_up_type_id(type_argument.type_id),
                            ),
                            insert_type(TypeInfo::ErrorRecovery),
                            warnings,
                            errors
                        );
                        let type_argument = TypeArgument {
                            type_id: new_type_id,
                            span: type_argument.span,
                        };
                        new_type_arguments.push(type_argument);
                    }
                    if !decl.type_parameters.is_empty() {
                        let new_decl = check!(
                            decl.monomorphize(&mut self[mod_path], &new_type_arguments, None),
                            return err(warnings, errors),
                            warnings,
                            errors
                        );
                        new_decl.type_id()
                    } else {
                        decl.type_id()
                    }
                }
                Some(TypedDeclaration::EnumDeclaration(decl)) => {
                    let mut new_type_arguments = vec![];
                    for type_argument in type_arguments.into_iter() {
                        let new_type_id = check!(
                            self.resolve_type_without_self(
                                mod_path,
                                &look_up_type_id(type_argument.type_id),
                            ),
                            insert_type(TypeInfo::ErrorRecovery),
                            warnings,
                            errors
                        );
                        let type_argument = TypeArgument {
                            type_id: new_type_id,
                            span: type_argument.span,
                        };
                        new_type_arguments.push(type_argument);
                    }
                    if !decl.type_parameters.is_empty() {
                        let new_decl = check!(
                            decl.monomorphize_with_type_arguments(
                                &mut self[mod_path],
                                &new_type_arguments,
                                None
                            ),
                            return err(warnings, errors),
                            warnings,
                            errors
                        );
                        new_decl.type_id()
                    } else {
                        decl.type_id()
                    }
                }
                _ => insert_type(TypeInfo::Unknown),
            },
            TypeInfo::Array(type_id, size) => {
                let elem_type_id = check!(
                    self.resolve_type_without_self(mod_path, &look_up_type_id(type_id)),
                    insert_type(TypeInfo::ErrorRecovery),
                    warnings,
                    errors
                );
                insert_type(TypeInfo::Array(elem_type_id, size))
            }
            TypeInfo::Ref(id) => id,
            o => insert_type(o),
        };
        ok(type_id, warnings, errors)
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
        r#type: TypeId,
        method_path: &Path,
        self_type: TypeId,
        args_buf: &VecDeque<TypedExpression>,
    ) -> CompileResult<TypedFunctionDeclaration> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let local_methods = self[mod_path].get_methods_for_type(r#type);
        let (method_name, method_prefix) = method_path.split_last().expect("method path is empty");

        // Ensure there's a module for the given method prefix.
        check!(
            self.check_submodule(method_prefix),
            return err(warnings, errors),
            warnings,
            errors
        );

        // This is a hack and I don't think it should be used.  We check the local namespace first,
        // but if nothing turns up then we try the namespace where the type itself is declared.
        let r#type = check!(
            self.resolve_type_with_self(
                method_prefix,
                look_up_type_id(r#type),
                self_type,
                method_name.span().clone(),
                false
            ),
            insert_type(TypeInfo::ErrorRecovery),
            warnings,
            errors
        );
        let mut ns_methods = self[method_prefix].get_methods_for_type(r#type);

        let mut methods = local_methods;
        methods.append(&mut ns_methods);

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
                        type_name: r#type.friendly_type_str(),
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
