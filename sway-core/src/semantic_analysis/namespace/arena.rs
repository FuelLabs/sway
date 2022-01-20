use crate::{CallPath, Visibility, error::*, semantic_analysis::{*, ast_node::*}, type_engine::*};
use generational_arena::{Arena, Index};
use lazy_static::lazy_static;
use std::{collections::VecDeque, sync::RwLock};
use sway_types::{Ident, Span, join_spans};
pub type NamespaceRef = Index;

pub(crate) trait NamespaceWrapper {
    /// this function either returns a struct (i.e. custom type), `None`, denoting the type that is
    /// being looked for is actually a generic, not-yet-resolved type.
    ///
    ///
    /// If a self type is given and anything on this ref chain refers to self, update the chain.
    fn resolve_type_with_self(&self, ty: TypeInfo, self_type: TypeId) -> Result<TypeId, ()>;
    fn resolve_type_without_self(&self, ty: &TypeInfo) -> TypeId;
    fn insert(&self, name: Ident, item: TypedDeclaration) -> CompileResult<()>;
    fn find_subfield_type(&self, subfield_exp: &[Ident]) -> CompileResult<(TypeId, TypeId)>;
    fn insert_module(&self, module_name: String, module_contents: Namespace);
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
    fn get_name_from_path(&self, path: &[Ident], name: &Ident) -> CompileResult<TypedDeclaration>;
    /// Used for calls that look like this:
    /// `foo::bar::function`
    /// where `foo` and `bar` are the prefixes
    /// and `function` is the suffix
    fn get_call_path(&self, symbol: &CallPath) -> CompileResult<TypedDeclaration>;
    fn get_symbol(&self, symbol: &Ident) -> CompileResult<TypedDeclaration> ;
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
    ) -> CompileResult<(Vec<OwnedTypedStructField>, String)> ;
     fn get_tuple_elems(
        &self,
        ty: TypeId,
        debug_string: impl Into<String>,
        debug_span: &Span,
    ) -> CompileResult<Vec<TypeId>>;
}

impl NamespaceWrapper for NamespaceRef {
     fn get_tuple_elems(&self, ty: TypeId, debug_string: impl Into<String>, debug_span: &Span) -> CompileResult<Vec<TypeId>> {
        read_module(|ns| ns.get_tuple_elems(ty, debug_string, debug_span), *self)
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
            } => Some(inner.clone()),
            _ => None,
        }
    }
fn get_symbol(&self, symbol: &Ident) -> CompileResult<TypedDeclaration> {
    let (path , true_symbol) = read_module(|m| {
        let empty = vec![];
        let path = m.use_synonyms.get(symbol).unwrap_or(&empty);
        let true_symbol =m 
            .use_aliases
            .get(&symbol.as_str().to_string())
            .unwrap_or(symbol);
            (path, true_symbol)
    }, *self);
        self.get_name_from_path(path, true_symbol)
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
                    .map(|decl| decl.clone())
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
            |m| {
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
        let r#type =  namespace.resolve_type_with_self(look_up_type_id(r#type), self_type)
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
        let ix = read_module(|m| m.modules.get(path[0].as_str()), *self);
        let mut ix: NamespaceRef = match ix {
            Some(ix) => *ix,
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
        for ident in path {
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
                //  if this is an enum or struct, import its implementations
                if decl.visibility() != Visibility::Public {
                    errors.push(CompileError::ImportPrivateSymbol {
                        name: item.as_str().to_string(),
                        span: item.span().clone(),
                    });
                }
                let a = decl.return_type().value;
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
                    |mut m| {
                        // no matter what, import it this way though.
                        match alias {
                            Some(alias) => {
                                m.use_synonyms.insert(alias.clone(), path);
                                m.use_aliases
                                    .insert(alias.as_str().to_string(), item.clone());
                            }
                            None => {
                                m.use_synonyms.insert(item.clone(), path);
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
            |ns| ns.insert_trait_implementation(trait_name, type_implementing_for, functions_buf),
            *self,
        )
    }
    fn insert_module(&self, module_name: String, module_contents: Namespace) {
        let ix = {
            let mut write_lock = MODULES.write().expect("poisoned lock");
            write_lock.insert(module_contents)
        };
        write_module(|mut ns| ns.insert_module(module_name, ix), *self)
    }

    fn find_subfield_type(&self, subfield_exp: &[Ident]) -> CompileResult<(TypeId, TypeId)> {
        read_module(|ns| ns.find_subfield_type(subfield_exp), *self)
    }
    fn insert(&self, name: Ident, item: TypedDeclaration) -> CompileResult<()> {
        write_module(|mut ns| ns.insert(name, item), *self)
    }
    fn resolve_type_with_self(&self, ty: TypeInfo, self_type: TypeId) -> Result<TypeId, ()> {
        let mut warnings = vec![];
        let mut errors = vec![];
        Ok(match ty {
            TypeInfo::Custom { ref name } => {
                match self.get_symbol(name).ok(&mut warnings, &mut errors) {
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
                    Some(TypedDeclaration::GenericTypeForFunctionScope { name, .. }) => {
                        crate::type_engine::insert_type(TypeInfo::UnknownGeneric {
                            name: name.as_str().to_string(),
                        })
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

pub fn create_module() -> NamespaceRef {
    let res = {
        let mut write_lock = MODULES.write().expect("poisoned mutex");
        write_lock.insert(Default::default())
    };
    res
}

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
pub fn write_module<F, R>(mut func: F, ix: NamespaceRef) -> R
where
    F: FnMut(&mut Namespace) -> R,
{
    let res = {
        let mut write_lock = MODULES.write().expect("poisoned lock");
        let mut ns = write_lock
            .get_mut(ix)
            .expect("namespace index did not exist in arena");
        func(ns)
    };
    res
}

lazy_static! {
    pub static ref MODULES: RwLock<Arena<Namespace>> = Default::default();
}
