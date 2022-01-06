use super::ast_node::{
    OwnedTypedStructField, TypedEnumDeclaration, TypedEnumVariant, TypedStructDeclaration,
    TypedStructField,
};
use crate::error::*;
use crate::parse_tree::Visibility;
use crate::semantic_analysis::TypedExpression;
use crate::span::Span;
use crate::type_engine::*;

use crate::CallPath;
use crate::{CompileResult, TypeInfo};
use crate::{Ident, TypedDeclaration, TypedFunctionDeclaration};
use std::collections::{BTreeMap, HashMap, VecDeque};

type ModuleName = String;
type TraitName = CallPath;

/// A namespace represents all items that exist either via declaration or importing.
#[derive(Clone, Debug, Default)]
pub struct Namespace {
    // This is a BTreeMap because we rely on its ordering being consistent. See
    // [Namespace::get_all_declared_symbols] -- we need that iterator to have a deterministic
    // order.
    symbols: BTreeMap<Ident, TypedDeclaration>,
    implemented_traits: HashMap<(TraitName, TypeInfo), Vec<TypedFunctionDeclaration>>,
    // Any other modules within this scope, where a module is a namespace associated with an identifier.
    // This is a BTreeMap because we rely on its ordering being consistent. See
    // [Namespace::get_all_imported_modules] -- we need that iterator to have a deterministic
    // order.
    modules: BTreeMap<ModuleName, Namespace>,
    use_synonyms: HashMap<Ident, Vec<Ident>>,
    // Represents an alternative name for a symbol.
    use_aliases: HashMap<String, Ident>,
}

impl Namespace {
    pub fn get_all_declared_symbols(&self) -> impl Iterator<Item = &TypedDeclaration> {
        self.symbols.values()
    }

    pub fn get_all_imported_modules(&self) -> impl Iterator<Item = &Namespace> {
        self.modules.values()
    }

    /// this function either returns a struct (i.e. custom type), `None`, denoting the type that is
    /// being looked for is actually a generic, not-yet-resolved type.
    ///
    ///
    /// If a self type is given and anything on this ref chain refers to self, update the chain.
    pub(crate) fn resolve_type_with_self(
        &self,
        ty: TypeInfo,
        self_type: TypeId,
    ) -> Result<TypeId, ()> {
        Ok(match ty {
            TypeInfo::Custom { ref name } => match self.get_symbol_by_str(name) {
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
            },
            TypeInfo::SelfType => self_type,
            TypeInfo::Ref(id) => id,
            o => insert_type(o),
        })
    }

    /// Used to resolve a type when there is no known self type. This is needed
    /// when declaring new self types.
    pub(crate) fn resolve_type_without_self(&self, ty: &TypeInfo) -> TypeId {
        let ty = ty.clone();
        match ty {
            TypeInfo::Custom { name } => match self.get_symbol_by_str(&name) {
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
            },
            TypeInfo::Ref(id) => id,
            o => insert_type(o),
        }
    }

    pub(crate) fn insert(&mut self, name: Ident, item: TypedDeclaration) -> CompileResult<()> {
        let mut warnings = vec![];
        let mut errors = vec![];
        if self.symbols.get(&name).is_some() {
            match item {
                TypedDeclaration::EnumDeclaration { .. }
                | TypedDeclaration::StructDeclaration { .. } => {
                    errors.push(CompileError::ShadowsOtherSymbol {
                        span: name.span().clone(),
                        name: name.as_str().to_string(),
                    });
                    return err(warnings, errors);
                }
                _ => {
                    warnings.push(CompileWarning {
                        span: name.span().clone(),
                        warning_content: Warning::ShadowsOtherSymbol {
                            name: name.span().as_str().to_string(),
                        },
                    });
                }
            }
        }
        self.symbols.insert(name.clone(), item.clone());
        ok((), warnings, errors)
    }

    // TODO(static span) remove this and switch to spans when we have arena spans
    pub(crate) fn get_symbol_by_str(&self, symbol: &str) -> Option<&TypedDeclaration> {
        let empty = vec![];
        let path = self
            .use_synonyms
            .iter()
            .find_map(|(name, value)| {
                if name.as_str() == symbol {
                    Some(value)
                } else {
                    None
                }
            })
            .unwrap_or(&empty);
        self.get_name_from_path_str(path, symbol).value
    }

    pub(crate) fn get_symbol(&self, symbol: &Ident) -> CompileResult<&TypedDeclaration> {
        let empty = vec![];
        let path = self.use_synonyms.get(symbol).unwrap_or(&empty);
        let true_symbol = self
            .use_aliases
            .get(&symbol.as_str().to_string())
            .unwrap_or(symbol);
        self.get_name_from_path(path, true_symbol)
    }

    /// Used for calls that look like this:
    /// `foo::bar::function`
    /// where `foo` and `bar` are the prefixes
    /// and `function` is the suffix
    pub(crate) fn get_call_path(&self, symbol: &CallPath) -> CompileResult<TypedDeclaration> {
        let path = if symbol.prefixes.is_empty() {
            self.use_synonyms
                .get(&symbol.suffix)
                .unwrap_or(&symbol.prefixes)
        } else {
            &symbol.prefixes
        };
        self.get_name_from_path(path, &symbol.suffix)
            .map(|decl| decl.clone())
    }

    fn get_name_from_path(&self, path: &[Ident], name: &Ident) -> CompileResult<&TypedDeclaration> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let module = check!(
            self.find_module_relative(path),
            return err(warnings, errors),
            warnings,
            errors
        );

        match module.symbols.get(name) {
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

    // TODO(static span) remove this when typeinfo uses spans
    fn get_name_from_path_str(
        &self,
        path: &[Ident],
        name: &str,
    ) -> CompileResult<&TypedDeclaration> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let module = check!(
            self.find_module_relative(path),
            return err(warnings, errors),
            warnings,
            errors
        );

        match module.symbols.iter().find_map(|(item, other)| {
            if item.as_str() == name {
                Some(other)
            } else {
                None
            }
        }) {
            Some(decl) => ok(decl, warnings, errors),
            None => {
                let span = match path.get(0) {
                    Some(ident) => ident.span().clone(),
                    None => {
                        errors.push(CompileError::Internal("Unable to construct span. This is a temporary error and will be fixed in a future release. )", Span { span: pest::Span::new(" ".into(), 0, 0).unwrap(),
                                path: None
                            }));
                        Span {
                            span: pest::Span::new(" ".into(), 0, 0).unwrap(),
                            path: None,
                        }
                    }
                };
                errors.push(CompileError::SymbolNotFound {
                    name: name.to_string(),
                    span,
                });
                err(warnings, errors)
            }
        }
    }

    pub(crate) fn find_module_relative(&self, path: &[Ident]) -> CompileResult<&Namespace> {
        let mut namespace = self;
        let mut errors = vec![];
        let warnings = vec![];
        for ident in path {
            match namespace.modules.get(ident.as_str()) {
                Some(o) => namespace = o,
                None => {
                    errors.push(CompileError::ModuleNotFound {
                        span: path.iter().fold(path[0].span().clone(), |acc, this_one| {
                            crate::utils::join_spans(acc, this_one.span().clone())
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
        ok(namespace, warnings, errors)
    }

    pub(crate) fn insert_trait_implementation(
        &mut self,
        trait_name: CallPath,
        type_implementing_for: TypeInfo,
        functions_buf: Vec<TypedFunctionDeclaration>,
    ) -> CompileResult<()> {
        let mut warnings = vec![];
        let errors = vec![];
        let new_prefixes = if trait_name.prefixes.is_empty() {
            self.use_synonyms
                .get(&trait_name.suffix)
                .unwrap_or(&trait_name.prefixes)
                .clone()
        } else {
            trait_name.prefixes
        };
        let trait_name = CallPath {
            suffix: trait_name.suffix,
            prefixes: new_prefixes,
        };
        if self
            .implemented_traits
            .insert((trait_name.clone(), type_implementing_for), functions_buf)
            .is_some()
        {
            warnings.push(CompileWarning {
                warning_content: Warning::OverridingTraitImplementation,
                span: trait_name.span(),
            })
        }
        ok((), warnings, errors)
    }

    pub fn insert_module(&mut self, module_name: String, module_contents: Namespace) {
        self.modules.insert(module_name, module_contents);
    }

    pub fn insert_dependency_module(&mut self, module_name: String, module_contents: Namespace) {
        self.modules.insert(module_name, module_contents);
    }

    pub(crate) fn find_enum(&self, enum_name: &Ident) -> Option<TypedEnumDeclaration> {
        match self.get_symbol(enum_name) {
            CompileResult {
                value: Some(TypedDeclaration::EnumDeclaration(inner)),
                ..
            } => Some(inner.clone()),
            _ => None,
        }
    }
    /// Returns a tuple where the first element is the [ResolvedType] of the actual expression,
    /// and the second is the [ResolvedType] of its parent, for control-flow analysis.
    pub(crate) fn find_subfield_type(
        &mut self,
        subfield_exp: &[Ident],
    ) -> CompileResult<(TypeId, TypeId)> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let mut ident_iter = subfield_exp.iter().peekable();
        let first_ident = ident_iter.next().unwrap();
        let symbol = match self.symbols.get(first_ident) {
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

    pub(crate) fn get_methods_for_type(&self, r#type: TypeId) -> Vec<TypedFunctionDeclaration> {
        let mut methods = vec![];
        let r#type = crate::type_engine::look_up_type_id(r#type);
        for ((_trait_name, type_info), l_methods) in &self.implemented_traits {
            if *type_info == r#type {
                methods.append(&mut l_methods.clone());
            }
        }
        methods
    }

    /// given a declaration that may refer to a variable which contains a struct,
    /// find that struct's fields and name for use in determining if a subfield expression is valid
    /// e.g. foo.bar.baz
    /// is foo a struct? does it contain a field bar? is foo.bar a struct? does foo.bar contain a
    /// field baz? this is the problem this function addresses
    pub(crate) fn get_struct_type_fields(
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

    pub(crate) fn get_tuple_elems(
        &self,
        ty: TypeId,
        debug_string: impl Into<String>,
        debug_span: &Span,
    ) -> CompileResult<Vec<TypeId>> {
        let warnings = vec![];
        let errors = vec![];
        let ty = crate::type_engine::look_up_type_id(ty);
        match ty {
            TypeInfo::Tuple(elems) => ok(elems, warnings, errors),
            TypeInfo::ErrorRecovery => err(warnings, errors),
            a => err(
                vec![],
                vec![CompileError::NotATuple {
                    name: debug_string.into(),
                    span: debug_span.clone(),
                    actually: a.friendly_type_str(),
                }],
            ),
        }
    }

    /// Given a path to a module, create synonyms to every symbol in that module.
    /// This is used when an import path contains an asterisk.
    pub(crate) fn star_import(
        &mut self,
        from_module: Option<&Namespace>,
        path: Vec<Ident>,
    ) -> CompileResult<()> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let namespace = { let base_namespace = match from_module {
                Some(base_namespace) => base_namespace,
                None => self,
            };
            check!(
                base_namespace.find_module_relative(&path),
                return err(warnings, errors),
                warnings,
                errors
            )
        };
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
        let implemented_traits = namespace.implemented_traits.clone();
        self.implemented_traits.extend(&mut implemented_traits.into_iter());
        for symbol in symbols {
            self.use_synonyms.insert(symbol, path.clone());
        }
        ok((), warnings, errors)
    }

    /// Pull a single item from a module and import it into this namespace.
    pub(crate) fn item_import(
        &mut self,
        from_namespace: Option<&Namespace>,
        path: Vec<Ident>,
        item: &Ident,
        alias: Option<Ident>,
    ) -> CompileResult<()> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let base_namespace = match from_namespace {
            Some(base_namespace) => base_namespace,
            None => self,
        };
        let namespace = check!(
            base_namespace.find_module_relative(&path),
            return err(warnings, errors),
            warnings,
            errors
        );
        let mut impls_to_insert = vec![];

        match namespace.symbols.get(item) {
            Some(decl) => {
                //  if this is an enum or struct, import its implementations
                if decl.visibility() != Visibility::Public {
                    errors.push(CompileError::ImportPrivateSymbol {
                        name: item.as_str().to_string(),
                        span: item.span().clone(),
                    });
                }
                let a = decl.return_type().value;
                namespace
                    .implemented_traits
                    .iter()
                    .filter(|((_trait_name, type_info), _impl)| {
                        a.map(look_up_type_id).as_ref() == Some(type_info)
                    })
                    .for_each(|(a, b)| {
                        impls_to_insert.push((a.clone(), b.to_vec()));
                    });
                // no matter what, import it this way though.
                match alias {
                    Some(alias) => {
                        self.use_synonyms.insert(alias.clone(), path);
                        self.use_aliases
                            .insert(alias.as_str().to_string(), item.clone());
                    }
                    None => {
                        self.use_synonyms.insert(item.clone(), path);
                    }
                };
            }
            None => {
                errors.push(CompileError::SymbolNotFound {
                    name: item.as_str().to_string(),
                    span: item.span().clone(),
                });
                return err(warnings, errors);
            }
        };

        impls_to_insert.into_iter().for_each(|(a, b)| {
            self.implemented_traits.insert(a, b);
        });

        ok((), warnings, errors)
    }

    /// Given a method and a type (plus a `self_type` to potentially resolve it), find that
    /// method in the namespace. Requires `args_buf` because of some special casing for the
    /// standard library where we pull the type from the arguments buffer.
    ///
    /// This function will generate a missing method error if the method is not found.
    pub(crate) fn find_method_for_type(
        &self,
        r#type: TypeId,
        method_name: &Ident,
        method_path: &[Ident],
        from_module: Option<&Namespace>,
        self_type: TypeId,
        args_buf: &VecDeque<TypedExpression>,
    ) -> CompileResult<TypedFunctionDeclaration> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let base_module = match from_module {
            Some(base_module) => base_module,
            None => self,
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
        let mut ns_methods = namespace.get_methods_for_type(r#type);

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
}
