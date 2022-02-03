use crate::{
    error::*,
    semantic_analysis::{ast_node::*, *},
    type_engine::*,
    CallPath, Visibility,
};
use generational_arena::{Arena, Index};
use lazy_static::lazy_static;
use std::{collections::VecDeque, sync::RwLock};
use sway_types::{join_spans, Ident, Span};
pub type NamespaceRef = Index;

pub trait NamespaceWrapper {
    /// this function either returns a struct (i.e. custom type), `None`, denoting the type that is
    /// being looked for is actually a generic, not-yet-resolved type.
    ///
    ///
    /// If a self type is given and anything on this ref chain refers to self, update the chain.
    #[allow(clippy::result_unit_err)]
    fn resolve_type_with_self(&self, ty: TypeInfo, self_type: TypeId) -> Result<TypeId, ()>;
    fn resolve_type_without_self(&self, ty: &TypeInfo) -> TypeId;
    fn insert(&self, name: Ident, item: TypedDeclaration) -> CompileResult<()>;
    fn insert_module(&self, module_name: String, module_contents: Namespace);
    fn insert_module_ref(&self, module_name: String, ix: NamespaceRef);
    fn insert_trait_implementation(
        &self,
        trait_name: CallPath,
        type_implementing_for: TypeInfo,
        functions_buf: Vec<TypedFunctionDeclaration>,
    ) -> CompileResult<()>;
    fn item_import(
        &self,
        from_namespace: Option<NamespaceRef>,
        path: Vec<Ident>,
        item: &Ident,
        alias: Option<Ident>,
    ) -> CompileResult<()>;
    fn find_module_relative(&self, path: &[Ident]) -> CompileResult<NamespaceRef>;
    /// Given a method and a type (plus a `self_type` to potentially resolve it), find that
    /// method in the namespace. Requires `args_buf` because of some special casing for the
    /// standard library where we pull the type from the arguments buffer.
    ///
    /// This function will generate a missing method error if the method is not found.
    fn find_method_for_type(
        &self,
        r#type: TypeId,
        method_name: &Ident,
        method_path: &[Ident],
        from_module: Option<NamespaceRef>,
        self_type: TypeId,
        args_buf: &VecDeque<TypedExpression>,
    ) -> CompileResult<TypedFunctionDeclaration>;

    /// Given a path to a module, create synonyms to every symbol in that module.
    /// This is used when an import path contains an asterisk.
    fn star_import(&self, from_module: Option<NamespaceRef>, path: Vec<Ident>)
        -> CompileResult<()>;
    fn get_methods_for_type(&self, r#type: TypeId) -> Vec<TypedFunctionDeclaration>;
    fn copy_methods_to_type(&self, old_type: TypeInfo, new_type: TypeInfo);
    fn get_name_from_path(&self, path: &[Ident], name: &Ident) -> CompileResult<TypedDeclaration>;
    /// Used for calls that look like this:
    /// `foo::bar::function`
    /// where `foo` and `bar` are the prefixes
    /// and `function` is the suffix
    fn get_call_path(&self, symbol: &CallPath) -> CompileResult<TypedDeclaration>;
    fn get_symbol(&self, symbol: &Ident) -> CompileResult<TypedDeclaration>;
    fn find_enum(&self, enum_name: &Ident) -> Option<TypedEnumDeclaration>;
    /// given a declaration that may refer to a variable which contains a struct,
    /// find that struct's fields and name for use in determining if a subfield expression is valid
    /// e.g. foo.bar.baz
    /// is foo a struct? does it contain a field bar? is foo.bar a struct? does foo.bar contain a
    /// field baz? this is the problem this function addresses
    fn get_struct_type_fields(
        &self,
        ty: TypeId,
        debug_string: impl Into<String>,
        debug_span: &Span,
    ) -> CompileResult<(Vec<OwnedTypedStructField>, String)>;
    fn get_tuple_elems(
        &self,
        ty: TypeId,
        debug_string: impl Into<String>,
        debug_span: &Span,
    ) -> CompileResult<Vec<TypeId>>;
    /// Returns a tuple where the first element is the [ResolvedType] of the actual expression,
    /// and the second is the [ResolvedType] of its parent, for control-flow analysis.
    fn find_subfield_type(&self, subfield_exp: &[Ident]) -> CompileResult<(TypeId, TypeId)>;
}

impl NamespaceWrapper for NamespaceRef {
    fn insert_module_ref(&self, module_name: String, ix: NamespaceRef) {
        write_module(|ns| ns.insert_module(module_name, ix), *self)
    }
    fn find_subfield_type(&self, subfield_exp: &[Ident]) -> CompileResult<(TypeId, TypeId)> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let mut ident_iter = subfield_exp.iter().peekable();
        let first_ident = ident_iter.next().unwrap();
        let symbol = match read_module(|m| m.symbols.get(first_ident).cloned(), *self) {
            Some(s) => s,
            None => {
                errors.push(CompileError::UnknownVariable {
                    var_name: first_ident.as_str().to_string(),
                    span: first_ident.span().clone(),
                });
                return err(warnings, errors);
            }
        };
        if ident_iter.peek().is_none() {
            let ty = check!(
                symbol.return_type(),
                return err(warnings, errors),
                warnings,
                errors
            );
            return ok((ty, ty), warnings, errors);
        }
        let mut symbol = check!(
            symbol.return_type(),
            return err(warnings, errors),
            warnings,
            errors
        );
        let mut type_fields =
            self.get_struct_type_fields(symbol, first_ident.as_str(), first_ident.span());
        warnings.append(&mut type_fields.warnings);
        errors.append(&mut type_fields.errors);
        let (mut fields, struct_name) = match type_fields.value {
            // if it is missing, the error message comes from within the above method
            // so we don't need to re-add it here
            None => return err(warnings, errors),
            Some(value) => value,
        };

        let mut parent_rover = symbol;

        for ident in ident_iter {
            // find the ident in the currently available fields
            let OwnedTypedStructField { r#type, .. } =
                match fields.iter().find(|x| x.name == ident.as_str()) {
                    Some(field) => field.clone(),
                    None => {
                        // gather available fields for the error message
                        let available_fields =
                            fields.iter().map(|x| x.name.as_str()).collect::<Vec<_>>();

                        errors.push(CompileError::FieldNotFound {
                            field_name: ident.clone(),
                            struct_name,
                            available_fields: available_fields.join(", "),
                            span: ident.span().clone(),
                        });
                        return err(warnings, errors);
                    }
                };

            match crate::type_engine::look_up_type_id(r#type) {
                TypeInfo::Struct {
                    fields: ref l_fields,
                    ..
                } => {
                    parent_rover = symbol;
                    fields = l_fields.clone();
                    symbol = r#type;
                }
                _ => {
                    fields = vec![];
                    parent_rover = symbol;
                    symbol = r#type;
                }
            }
        }
        ok((symbol, parent_rover), warnings, errors)
    }
    fn get_tuple_elems(
        &self,
        ty: TypeId,
        debug_string: impl Into<String>,
        debug_span: &Span,
    ) -> CompileResult<Vec<TypeId>> {
        let debug_string = debug_string.into();
        read_module(
            |ns| ns.get_tuple_elems(ty, debug_string.clone(), debug_span),
            *self,
        )
    }
    fn get_struct_type_fields(
        &self,
        ty: TypeId,
        debug_string: impl Into<String>,
        debug_span: &Span,
    ) -> CompileResult<(Vec<OwnedTypedStructField>, String)> {
        let ty = crate::type_engine::look_up_type_id(ty);
        match ty {
            TypeInfo::Struct { name, fields } => ok((fields.to_vec(), name), vec![], vec![]),
            // If we hit `ErrorRecovery` then the source of that type should have populated
            // the error buffer elsewhere
            TypeInfo::ErrorRecovery => err(vec![], vec![]),
            a => err(
                vec![],
                vec![CompileError::NotAStruct {
                    name: debug_string.into(),
                    span: debug_span.clone(),
                    actually: a.friendly_type_str(),
                }],
            ),
        }
    }
    fn find_enum(&self, enum_name: &Ident) -> Option<TypedEnumDeclaration> {
        match self.get_symbol(enum_name) {
            CompileResult {
                value: Some(TypedDeclaration::EnumDeclaration(inner)),
                ..
            } => Some(inner),
            _ => None,
        }
    }
    fn get_symbol(&self, symbol: &Ident) -> CompileResult<TypedDeclaration> {
        let (path, true_symbol) = read_module(
            |m| {
                let empty = vec![];
                let path = m.use_synonyms.get(symbol).unwrap_or(&empty);
                let true_symbol = m
                    .use_aliases
                    .get(&symbol.as_str().to_string())
                    .unwrap_or(symbol);
                (path.clone(), true_symbol.clone())
            },
            *self,
        );
        self.get_name_from_path(&path, &true_symbol)
    }
    fn get_call_path(&self, symbol: &CallPath) -> CompileResult<TypedDeclaration> {
        read_module(
            |m| {
                let path = if symbol.prefixes.is_empty() {
                    m.use_synonyms
                        .get(&symbol.suffix)
                        .unwrap_or(&symbol.prefixes)
                } else {
                    &symbol.prefixes
                };
                self.get_name_from_path(path, &symbol.suffix)
            },
            *self,
        )
    }
    fn get_name_from_path(&self, path: &[Ident], name: &Ident) -> CompileResult<TypedDeclaration> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let module = check!(
            self.find_module_relative(path),
            return err(warnings, errors),
            warnings,
            errors
        );
        match read_module(|module| module.symbols.get(name).cloned(), module) {
            Some(decl) => ok(decl, warnings, errors),
            None => {
                errors.push(CompileError::SymbolNotFound {
                    name: name.as_str().to_string(),
                    span: name.span().clone(),
                });
                err(warnings, errors)
            }
        }
    }
    fn get_methods_for_type(&self, r#type: TypeId) -> Vec<TypedFunctionDeclaration> {
        read_module(|ns| ns.get_methods_for_type(r#type), *self)
    }
    fn copy_methods_to_type(&self, old_type: TypeInfo, new_type: TypeInfo) {
        write_module(
            move |ns| ns.copy_methods_to_type(old_type.clone(), new_type),
            *self,
        )
    }

    fn star_import(
        &self,
        from_module: Option<NamespaceRef>,
        path: Vec<Ident>,
    ) -> CompileResult<()> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let namespace = {
            let base_namespace = match from_module {
                Some(base_namespace) => base_namespace,
                None => *self,
            };
            check!(
                base_namespace.find_module_relative(&path),
                return err(warnings, errors),
                warnings,
                errors
            )
        };
        let (symbols, implemented_traits) = read_module(
            |namespace| {
                let symbols = namespace
                    .symbols
                    .iter()
                    .filter_map(|(symbol, decl)| {
                        if decl.visibility() == Visibility::Public {
                            Some(symbol.clone())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                (symbols, namespace.implemented_traits.clone())
            },
            namespace,
        );
        write_module(
            move |m| {
                m.implemented_traits
                    .extend(&mut implemented_traits.into_iter());
                for symbol in symbols {
                    m.use_synonyms.insert(symbol, path.clone());
                }
            },
            *self,
        );
        ok((), warnings, errors)
    }
    fn find_method_for_type(
        &self,
        r#type: TypeId,
        method_name: &Ident,
        method_path: &[Ident],
        from_module: Option<NamespaceRef>,
        self_type: TypeId,
        args_buf: &VecDeque<TypedExpression>,
    ) -> CompileResult<TypedFunctionDeclaration> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let base_module = match from_module {
            Some(base_module) => base_module,
            None => *self,
        };
        let namespace = check!(
            base_module.find_module_relative(method_path),
            return err(warnings, errors),
            warnings,
            errors
        );

        // This is a hack and I don't think it should be used.  We check the local namespace first,
        // but if nothing turns up then we try the namespace where the type itself is declared.
        let r#type = namespace
            .resolve_type_with_self(look_up_type_id(r#type), self_type)
            .unwrap_or_else(|_| {
                errors.push(CompileError::UnknownType {
                    span: method_name.span().clone(),
                });
                insert_type(TypeInfo::ErrorRecovery)
            });
        let local_methods = self.get_methods_for_type(r#type);
        let mut ns_methods = read_module(
            |namespace| namespace.get_methods_for_type(r#type),
            namespace,
        );

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
                        method_name: method_name.as_str().to_string(),
                        type_name: r#type.friendly_type_str(),
                        span: method_name.span().clone(),
                    });
                }
                err(warnings, errors)
            }
        }
    }
    fn find_module_relative(&self, path: &[Ident]) -> CompileResult<NamespaceRef> {
        let mut errors = vec![];
        let warnings = vec![];
        if path.is_empty() {
            return ok(*self, warnings, errors);
        }
        let ix = read_module(|m| m.modules.get(path[0].as_str()).cloned(), *self);
        let mut ix: NamespaceRef = match ix {
            Some(ix) => ix,
            None => {
                errors.push(CompileError::ModuleNotFound {
                    span: path.iter().fold(path[0].span().clone(), |acc, this_one| {
                        join_spans(acc, this_one.span().clone())
                    }),
                    name: path
                        .iter()
                        .map(|x| x.as_str())
                        .collect::<Vec<_>>()
                        .join("::"),
                });
                return err(warnings, errors);
            }
        };
        for ident in path.iter().skip(1) {
            match read_module(
                |namespace| namespace.modules.get(ident.as_str()).cloned(),
                ix,
            ) {
                Some(ns_ix) => {
                    ix = ns_ix;
                }
                _ => {
                    errors.push(CompileError::ModuleNotFound {
                        span: path.iter().fold(path[0].span().clone(), |acc, this_one| {
                            join_spans(acc, this_one.span().clone())
                        }),
                        name: path
                            .iter()
                            .map(|x| x.as_str())
                            .collect::<Vec<_>>()
                            .join("::"),
                    });
                    return err(warnings, errors);
                }
            };
        }

        ok(ix, warnings, errors)
    }
    /// Pull a single item from a module and import it into this namespace.
    fn item_import(
        &self,
        from_namespace: Option<NamespaceRef>,
        path: Vec<Ident>,
        item: &Ident,
        alias: Option<Ident>,
    ) -> CompileResult<()> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let base_namespace = match from_namespace {
            Some(base_namespace) => base_namespace,
            None => *self,
        };
        let namespace = check!(
            base_namespace.find_module_relative(&path),
            return err(warnings, errors),
            warnings,
            errors
        );
        let mut impls_to_insert = vec![];

        match read_module(|namespace| namespace.symbols.get(item).cloned(), namespace) {
            Some(decl) => {
                if decl.visibility() != Visibility::Public {
                    errors.push(CompileError::ImportPrivateSymbol {
                        name: item.as_str().to_string(),
                        span: item.span().clone(),
                    });
                }
                // if this is a const, insert it into the local namespace directly
                if let TypedDeclaration::VariableDeclaration(TypedVariableDeclaration {
                    is_mutable: VariableMutability::ExportedConst,
                    ref name,
                    ..
                }) = decl
                {
                    self.insert(alias.unwrap_or_else(|| name.clone()), decl.clone());
                    return ok((), warnings, errors);
                }
                let a = decl.return_type().value;
                //  if this is an enum or struct, import its implementations
                let mut res = read_module(
                    move |namespace| {
                        namespace
                            .implemented_traits
                            .iter()
                            .filter(|((_trait_name, type_info), _impl)| {
                                a.map(look_up_type_id).as_ref() == Some(type_info)
                            })
                            .fold(Vec::new(), |mut acc, (a, b)| {
                                acc.push((a.clone(), b.to_vec()));
                                acc
                            })
                    },
                    namespace,
                );
                impls_to_insert.append(&mut res);
                write_module(
                    |m| {
                        // no matter what, import it this way though.
                        match alias.clone() {
                            Some(alias) => {
                                m.use_synonyms.insert(alias.clone(), path.clone());
                                m.use_aliases
                                    .insert(alias.as_str().to_string(), item.clone());
                            }
                            None => {
                                m.use_synonyms.insert(item.clone(), path.clone());
                            }
                        };
                    },
                    *self,
                );
            }
            None => {
                errors.push(CompileError::SymbolNotFound {
                    name: item.as_str().to_string(),
                    span: item.span().clone(),
                });
                return err(warnings, errors);
            }
        };

        write_module(
            |m| {
                impls_to_insert.into_iter().for_each(|(a, b)| {
                    m.implemented_traits.insert(a, b);
                });
            },
            *self,
        );

        ok((), warnings, errors)
    }
    fn insert_trait_implementation(
        &self,
        trait_name: CallPath,
        type_implementing_for: TypeInfo,
        functions_buf: Vec<TypedFunctionDeclaration>,
    ) -> CompileResult<()> {
        write_module(
            move |ns| {
                ns.insert_trait_implementation(trait_name, type_implementing_for, functions_buf)
            },
            *self,
        )
    }
    fn insert_module(&self, module_name: String, module_contents: Namespace) {
        let ix = {
            let mut write_lock = MODULES.write().expect("poisoned lock");
            write_lock.insert(module_contents)
        };
        write_module(|ns| ns.insert_module(module_name, ix), *self)
    }

    fn insert(&self, name: Ident, item: TypedDeclaration) -> CompileResult<()> {
        write_module(|ns| ns.insert(name, item), *self)
    }
    fn resolve_type_with_self(&self, ty: TypeInfo, self_type: TypeId) -> Result<TypeId, ()> {
        let mut warnings = vec![];
        let mut errors = vec![];
        Ok(match ty {
            TypeInfo::Custom { ref name } => {
                match self.get_symbol(name).ok(&mut warnings, &mut errors) {
                    Some(TypedDeclaration::StructDeclaration(decl)) => {
                        let old_struct = TypeInfo::Struct {
                            name: decl.name.as_str().to_string(),
                            fields: decl
                                .fields
                                .iter()
                                .map(TypedStructField::as_owned_typed_struct_field)
                                .collect::<Vec<_>>(),
                        };
                        let mut new_struct = old_struct.clone();
                        if !decl.type_parameters.is_empty() {
                            let new_decl = decl.monomorphize();
                            new_struct = TypeInfo::Struct {
                                name: new_decl.name.as_str().to_string(),
                                fields: new_decl
                                    .fields
                                    .iter()
                                    .map(TypedStructField::as_owned_typed_struct_field)
                                    .collect::<Vec<_>>(),
                            };
                            self.copy_methods_to_type(old_struct, new_struct.clone());
                        }
                        crate::type_engine::insert_type(new_struct)
                    }
                    Some(TypedDeclaration::EnumDeclaration(decl)) => {
                        let old_enum = TypeInfo::Enum {
                            name: decl.name.as_str().to_string(),
                            variant_types: decl
                                .variants
                                .iter()
                                .map(TypedEnumVariant::as_owned_typed_enum_variant)
                                .collect(),
                        };
                        let mut new_enum = old_enum.clone();
                        if !decl.type_parameters.is_empty() {
                            let new_decl = decl.monomorphize();
                            new_enum = TypeInfo::Enum {
                                name: new_decl.name.as_str().to_string(),
                                variant_types: new_decl
                                    .variants
                                    .iter()
                                    .map(TypedEnumVariant::as_owned_typed_enum_variant)
                                    .collect(),
                            };
                            self.copy_methods_to_type(old_enum, new_enum.clone());
                        }
                        crate::type_engine::insert_type(new_enum)
                    }
                    Some(TypedDeclaration::GenericTypeForFunctionScope { name, .. }) => {
                        crate::type_engine::insert_type(TypeInfo::UnknownGeneric { name })
                    }
                    _ => return Err(()),
                }
            }
            TypeInfo::SelfType => self_type,
            TypeInfo::Ref(id) => id,
            o => insert_type(o),
        })
    }
    fn resolve_type_without_self(&self, ty: &TypeInfo) -> TypeId {
        let ty = ty.clone();
        let mut warnings = vec![];
        let mut errors = vec![];
        match ty {
            TypeInfo::Custom { name } => {
                match self.get_symbol(&name).ok(&mut warnings, &mut errors) {
                    Some(TypedDeclaration::StructDeclaration(TypedStructDeclaration {
                        name,
                        fields,
                        ..
                    })) => crate::type_engine::insert_type(TypeInfo::Struct {
                        name: name.as_str().to_string(),
                        fields: fields
                            .iter()
                            .map(TypedStructField::as_owned_typed_struct_field)
                            .collect::<Vec<_>>(),
                    }),
                    Some(TypedDeclaration::EnumDeclaration(TypedEnumDeclaration {
                        name,
                        variants,
                        ..
                    })) => crate::type_engine::insert_type(TypeInfo::Enum {
                        name: name.as_str().to_string(),
                        variant_types: variants
                            .iter()
                            .map(TypedEnumVariant::as_owned_typed_enum_variant)
                            .collect(),
                    }),
                    _ => crate::type_engine::insert_type(TypeInfo::Unknown),
                }
            }
            TypeInfo::Ref(id) => id,
            o => insert_type(o),
        }
    }
}

/// Create a new module ([Namespace]), insert it into the arena, and get its id back.
pub fn create_module() -> NamespaceRef {
    let res = {
        let mut write_lock = MODULES.write().expect("poisoned mutex");
        write_lock.insert(Default::default())
    };
    res
}

/// Given a function `func` and a reference to a module `ix`, read from `MODULES[ix]` with `func`.
pub fn read_module<F, R>(mut func: F, ix: NamespaceRef) -> R
where
    F: FnMut(&Namespace) -> R,
{
    let res = {
        let read_lock = MODULES.read().expect("poisoned lock");
        let ns = read_lock
            .get(ix)
            .expect("namespace index did not exist in arena");
        func(ns)
    };
    res
}

/// Given a function `func` and a reference to a module `ix`, mutate `MODULES[ix]` with `func`.
pub fn write_module<F, R>(func: F, ix: NamespaceRef) -> R
where
    F: FnOnce(&mut Namespace) -> R,
{
    let res = {
        let mut write_lock = MODULES.write().expect("poisoned lock");
        let ns = write_lock
            .get_mut(ix)
            .expect("namespace index did not exist in arena");
        func(ns)
    };
    res
}

lazy_static! {
    /// The arena which contains all modules in all dependencies and the main compilation target.
    pub static ref MODULES: RwLock<Arena<Namespace>> = Default::default();
}

/// Given a [NamespaceRef], get a clone of the actual [Namespace] it refers to.
pub fn retrieve_module(ix: NamespaceRef) -> Namespace {
    let module = {
        let lock = MODULES.read().expect("poisoned lock");
        lock.get(ix)
            .expect("index did not exist in namespace arena")
            .clone()
    };
    module
}

/// Given a [NamespaceRef] that refers to a module, construct a new `Namespace` (incurring the
/// cloning cost) with `parent` as its parent.
pub fn create_new_scope(parent: NamespaceRef) -> NamespaceRef {
    let new_module = read_module(|ns| ns.clone(), parent);

    let res = {
        let mut write_lock = MODULES.write().expect("poisoned mutex");
        write_lock.insert(new_module)
    };
    res
}
