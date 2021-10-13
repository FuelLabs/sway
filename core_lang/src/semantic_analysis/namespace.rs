use super::ast_node::{TypedEnumDeclaration, TypedStructDeclaration, TypedStructField};
use crate::error::*;
use crate::parse_tree::MethodName;
use crate::semantic_analysis::TypedExpression;
use crate::span::Span;
use crate::types::{MaybeResolvedType, PartiallyResolvedType, ResolvedType};
use crate::CallPath;
use crate::{CompileResult, TypeInfo};
use crate::{Ident, TypedDeclaration, TypedFunctionDeclaration};
use std::collections::{HashMap, VecDeque};

type ModuleName = String;
type TraitName<'a> = CallPath<'a>;

#[derive(Clone, Debug, Default)]
pub struct Namespace<'sc> {
    pub(crate) symbols: HashMap<Ident<'sc>, TypedDeclaration<'sc>>,
    pub(crate) implemented_traits:
        HashMap<(TraitName<'sc>, MaybeResolvedType<'sc>), Vec<TypedFunctionDeclaration<'sc>>>,
    /// any imported namespaces associated with an ident which is a  library name
    pub(crate) modules: HashMap<ModuleName, Namespace<'sc>>,
    /// The crate namespace, to be used in absolute importing. This is `None` if the current
    /// namespace _is_ the root namespace.
    pub(crate) crate_namespace: Box<Option<Namespace<'sc>>>,
    use_synonyms: HashMap<Ident<'sc>, Vec<Ident<'sc>>>,
    use_aliases: HashMap<String, Ident<'sc>>,
}

impl<'sc> Namespace<'sc> {
    /// this function either returns a struct (i.e. custom type), `None`, denoting the type that is
    /// being looked for is actually a generic, not-yet-resolved type.
    pub(crate) fn resolve_type(
        &self,
        ty: &TypeInfo<'sc>,
        self_type: &MaybeResolvedType<'sc>,
    ) -> MaybeResolvedType<'sc> {
        let ty = ty.clone();
        match ty {
            TypeInfo::Custom { name } => match self.get_symbol(&name) {
                Some(TypedDeclaration::StructDeclaration(TypedStructDeclaration {
                    name,
                    fields,
                    ..
                })) => MaybeResolvedType::Resolved(ResolvedType::Struct {
                    name: name.clone(),
                    fields: fields.clone(),
                }),
                Some(TypedDeclaration::EnumDeclaration(TypedEnumDeclaration {
                    name,
                    variants,
                    ..
                })) => MaybeResolvedType::Resolved(ResolvedType::Enum {
                    name: name.clone(),
                    variant_types: variants.iter().map(|x| x.r#type.clone()).collect(),
                }),
                Some(_) => MaybeResolvedType::Partial(PartiallyResolvedType::Generic {
                    name: name.clone(),
                }),
                None => MaybeResolvedType::Partial(PartiallyResolvedType::Generic {
                    name: name.clone(),
                }),
            },
            TypeInfo::SelfType => self_type.clone(),

            o => o.to_resolved(),
        }
    }
    /// Used to resolve a type when there is no known self type. This is needed
    /// when declaring new self types.
    pub(crate) fn resolve_type_without_self(&self, ty: &TypeInfo<'sc>) -> MaybeResolvedType<'sc> {
        let ty = ty.clone();
        match ty {
            TypeInfo::Custom { name } => match self.get_symbol(&name) {
                Some(TypedDeclaration::StructDeclaration(TypedStructDeclaration {
                    name,
                    fields,
                    ..
                })) => MaybeResolvedType::Resolved(ResolvedType::Struct {
                    name: name.clone(),
                    fields: fields.clone(),
                }),
                Some(TypedDeclaration::EnumDeclaration(TypedEnumDeclaration {
                    name,
                    variants,
                    ..
                })) => MaybeResolvedType::Resolved(ResolvedType::Enum {
                    name: name.clone(),
                    variant_types: variants.iter().map(|x| x.r#type.clone()).collect(),
                }),
                Some(_) => MaybeResolvedType::Partial(PartiallyResolvedType::Generic {
                    name: name.clone(),
                }),
                None => MaybeResolvedType::Partial(PartiallyResolvedType::Generic {
                    name: name.clone(),
                }),
            },
            TypeInfo::SelfType => MaybeResolvedType::Partial(PartiallyResolvedType::SelfType),
            o => o.to_resolved(),
        }
    }
    /// Given a path to a module, create synonyms to every symbol in that module.
    /// This is used when an import path contains an asterisk.
    pub(crate) fn star_import(
        &mut self,
        path: Vec<Ident<'sc>>,
        is_absolute: bool,
    ) -> CompileResult<'sc, ()> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let namespace = check!(
            self.find_module(&path, is_absolute),
            return err(warnings, errors),
            warnings,
            errors
        );
        for (symbol, _) in namespace.symbols.clone() {
            self.use_synonyms.insert(symbol, path.clone());
        }
        ok((), warnings, errors)
    }

    /// Pull a single item from a module and import it into this namespace.
    pub(crate) fn item_import(
        &mut self,
        path: Vec<Ident<'sc>>,
        item: &Ident<'sc>,
        is_absolute: bool,
        alias: Option<Ident<'sc>>
    ) -> CompileResult<'sc, ()> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let namespace = check!(
            self.find_module(&path, is_absolute),
            return err(warnings, errors),
            warnings,
            errors
        );

        match namespace.symbols.get(item) {
            Some(_) => {
                match alias {
                    Some(alias) => {
                        self.use_synonyms.insert(alias.clone(), path);
                        self.use_aliases.insert(alias.primary_name.to_string(), item.clone());
                    },
                    None => {
                        self.use_synonyms.insert(item.clone(), path);
                    }
                }
            }
            None => {
                errors.push(CompileError::SymbolNotFound {
                    name: item.primary_name,
                    span: item.span.clone(),
                });
                return err(warnings, errors);
            }
        };

        ok((), warnings, errors)
    }

    pub(crate) fn merge_namespaces(&mut self, other: &Namespace<'sc>) {
        for (name, symbol) in &other.symbols {
            self.symbols.insert(name.clone(), symbol.clone());
        }
        for ((name, typ), trait_impl) in &other.implemented_traits {
            self.implemented_traits
                .insert((name.clone(), typ.clone()), trait_impl.clone());
        }

        for (mod_name, namespace) in &other.modules {
            self.modules.insert(mod_name.clone(), namespace.clone());
        }
    }

    pub(crate) fn insert(
        &mut self,
        name: Ident<'sc>,
        item: TypedDeclaration<'sc>,
    ) -> CompileResult<()> {
        let mut warnings = vec![];
        if self.symbols.get(&name).is_some() {
            warnings.push(CompileWarning {
                span: name.span.clone(),
                warning_content: Warning::OverridesOtherSymbol {
                    name: name.clone().span.str(),
                },
            });
        }
        self.symbols.insert(name.clone(), item.clone());
        ok((), warnings, vec![])
    }

    pub(crate) fn get_symbol(&self, symbol: &Ident<'sc>) -> Option<&TypedDeclaration<'sc>> {
        let empty = vec![];
        let path = self.use_synonyms.get(symbol).unwrap_or(&empty);
        let true_symbol = self.use_aliases.get(&symbol.primary_name.to_string()).unwrap_or(symbol);
        self.get_name_from_path(path, true_symbol).value
    }

    /// Used for calls that look like this:
    /// `foo::bar::function`
    /// where `foo` and `bar` are the prefixes
    /// and `function` is the suffix
    pub(crate) fn get_call_path(
        &self,
        symbol: &CallPath<'sc>,
    ) -> CompileResult<'sc, TypedDeclaration<'sc>> {
        let path = if symbol.prefixes.is_empty() {
            self.use_synonyms
                .get(&symbol.suffix)
                .unwrap_or(&symbol.prefixes)
        } else {
            &symbol.prefixes
        };
        self.get_name_from_path(&path, &symbol.suffix)
            .map(|decl| decl.clone())
    }

    fn get_name_from_path(
        &self,
        path: &[Ident<'sc>],
        name: &Ident<'sc>,
    ) -> CompileResult<'sc, &TypedDeclaration<'sc>> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let module = check!(
            self.find_module(path, false),
            return err(warnings, errors),
            warnings,
            errors
        );

        match module.symbols.get(name) {
            Some(decl) => ok(decl, warnings, errors),
            None => {
                errors.push(CompileError::SymbolNotFound {
                    name: name.primary_name,
                    span: name.span.clone(),
                });
                err(warnings, errors)
            }
        }
    }

    pub(crate) fn find_module(
        &self,
        path: &[Ident<'sc>],
        is_absolute: bool,
    ) -> CompileResult<'sc, &Namespace<'sc>> {
        let mut namespace = if is_absolute {
            if let Some(ns) = &*self.crate_namespace {
                // this is an absolute import and this is a submodule, so we want the
                // crate global namespace here
                ns
            } else {
                // this is an absolute import and we are in the root module, so we want
                // this namespace
                self
            }
        } else {
            self
        };
        let mut errors = vec![];
        let warnings = vec![];
        for ident in path {
            match namespace.modules.get(ident.primary_name) {
                Some(o) => namespace = o,
                None => {
                    errors.push(CompileError::ModuleNotFound {
                        span: path.iter().fold(path[0].span.clone(), |acc, this_one| {
                            crate::utils::join_spans(acc, this_one.span.clone())
                        }),
                        name: path
                            .iter()
                            .map(|x| x.primary_name)
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
        trait_name: CallPath<'sc>,
        type_implementing_for: MaybeResolvedType<'sc>,
        functions_buf: Vec<TypedFunctionDeclaration<'sc>>,
    ) -> CompileResult<()> {
        let mut warnings = vec![];
        let errors = vec![];
        let new_prefixes = if trait_name.prefixes.is_empty() {
            self.use_synonyms
                .get(&trait_name.suffix)
                .unwrap_or_else(|| &trait_name.prefixes)
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

    pub fn insert_module(&mut self, module_name: String, module_contents: Namespace<'sc>) {
        self.modules.insert(module_name, module_contents);
    }
    pub fn insert_dependency_module(
        &mut self,
        module_name: String,
        module_contents: Namespace<'sc>,
    ) {
        self.modules.insert(
            module_name,
            module_contents.modules.into_iter().next().unwrap().1,
        );
    }
    pub(crate) fn find_enum(&self, enum_name: &Ident<'sc>) -> Option<TypedEnumDeclaration<'sc>> {
        match self.get_symbol(enum_name) {
            Some(TypedDeclaration::EnumDeclaration(inner)) => Some(inner.clone()),
            _ => None,
        }
    }
    /// Returns a tuple where the first element is the [ResolvedType] of the actual expression,
    /// and the second is the [ResolvedType] of its parent, for control-flow analysis.
    pub(crate) fn find_subfield_type(
        &self,
        subfield_exp: &[Ident<'sc>],
    ) -> CompileResult<'sc, (MaybeResolvedType<'sc>, MaybeResolvedType<'sc>)> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let mut ident_iter = subfield_exp.iter().peekable();
        let first_ident = ident_iter.next().unwrap();
        let symbol = match self.symbols.get(first_ident) {
            Some(s) => s,
            None => {
                errors.push(CompileError::UnknownVariable {
                    var_name: first_ident.primary_name.to_string(),
                    span: first_ident.span.clone(),
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
            return ok((ty.clone(), ty), warnings, errors);
        }
        let mut symbol = check!(
            symbol.return_type(),
            return err(warnings, errors),
            warnings,
            errors
        );
        let mut type_fields =
            self.get_struct_type_fields(&symbol, first_ident.primary_name, &first_ident.span);
        warnings.append(&mut type_fields.warnings);
        errors.append(&mut type_fields.errors);
        let (mut fields, struct_name) = match type_fields.value {
            // if it is missing, the error message comes from within the above method
            // so we don't need to re-add it here
            None => return err(warnings, errors),
            Some(value) => value,
        };

        let mut parent_rover = symbol.clone();

        for ident in ident_iter {
            // find the ident in the currently available fields
            let TypedStructField { r#type, .. } = match fields.iter().find(|x| x.name == *ident) {
                Some(field) => field.clone(),
                None => {
                    // gather available fields for the error message
                    let field_name = &(*ident.primary_name);
                    let available_fields = fields
                        .iter()
                        .map(|x| &(*x.name.primary_name))
                        .collect::<Vec<_>>();

                    errors.push(CompileError::FieldNotFound {
                        field_name,
                        struct_name: &(*struct_name.primary_name),
                        available_fields: available_fields.join(", "),
                        span: ident.span.clone(),
                    });
                    return err(warnings, errors);
                }
            };
            match r#type {
                ResolvedType::Struct {
                    fields: ref l_fields,
                    ..
                } => {
                    parent_rover = symbol.clone();
                    fields = l_fields.clone();
                    symbol = MaybeResolvedType::Resolved(r#type);
                }
                _ => {
                    fields = vec![];
                    parent_rover = symbol.clone();
                    symbol = MaybeResolvedType::Resolved(r#type);
                }
            }
        }
        ok((symbol, parent_rover), warnings, errors)
    }

    pub(crate) fn get_methods_for_type(
        &self,
        r#type: &MaybeResolvedType<'sc>,
    ) -> Vec<TypedFunctionDeclaration<'sc>> {
        let mut methods = vec![];
        for ((_trait_name, type_info), l_methods) in &self.implemented_traits {
            if type_info == r#type {
                methods.append(&mut l_methods.clone());
            }
        }
        methods
    }

    /// Given a method and a type (plus a `self_type` to potentially resolve it), find that
    /// method in the namespace. Requires `args_buf` because of some special casing for the
    /// standard library where we pull the type from the arguments buffer.
    ///
    /// This function will generate a missing method error if the method is not found.
    pub(crate) fn find_method_for_type(
        &self,
        r#type: &MaybeResolvedType<'sc>,
        method_name: &MethodName<'sc>,
        self_type: &MaybeResolvedType<'sc>,
        args_buf: &VecDeque<TypedExpression<'sc>>,
    ) -> CompileResult<'sc, TypedFunctionDeclaration<'sc>> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let (namespace, method_name, r#type) = match method_name {
            // something like a.b(c)
            MethodName::FromModule { ref method_name } => (self, method_name, r#type.clone()),
            // something like blah::blah::~Type::foo()
            MethodName::FromType {
                ref call_path,
                ref type_name,
                ref is_absolute,
            } => {
                let module = check!(
                    self.find_module(&call_path.prefixes[..], *is_absolute),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let r#type = if let Some(type_name) = type_name {
                    module.resolve_type(type_name, self_type)
                } else {
                    args_buf[0].return_type.clone()
                };
                (module, &call_path.suffix, r#type)
            }
        };

        // This is a hack and I don't think it should be used.  We check the local namespace first,
        // but if nothing turns up then we try the namespace where the type itself is declared.
        let methods = self.get_methods_for_type(&r#type);
        let methods = match methods[..] {
            [] => namespace.get_methods_for_type(&r#type),
            _ => methods,
        };

        match methods
            .into_iter()
            .find(|TypedFunctionDeclaration { name, .. }| name == method_name)
        {
            Some(o) => ok(o, warnings, errors),
            None => {
                if args_buf.get(0).map(|x| &x.return_type)
                    != Some(&MaybeResolvedType::Resolved(ResolvedType::ErrorRecovery))
                {
                    errors.push(CompileError::MethodNotFound {
                        method_name: method_name.primary_name.to_string(),
                        type_name: args_buf[0].return_type.friendly_type_str(),
                        span: method_name.span.clone(),
                    });
                }
                err(warnings, errors)
            }
        }
    }
    /// given a declaration that may refer to a variable which contains a struct,
    /// find that struct's fields and name for use in determining if a subfield expression is valid
    /// e.g. foo.bar.baz
    /// is foo a struct? does it contain a field bar? is foo.bar a struct? does foo.bar contain a
    /// field baz? this is the problem this function addresses
    pub(crate) fn get_struct_type_fields(
        &self,
        ty: &MaybeResolvedType<'sc>,
        debug_string: impl Into<String>,
        debug_span: &Span<'sc>,
    ) -> CompileResult<'sc, (Vec<TypedStructField<'sc>>, Ident<'sc>)> {
        match ty {
            MaybeResolvedType::Resolved(ResolvedType::Struct { name, fields }) => {
                ok((fields.to_vec(), name.clone()), vec![], vec![])
            }
            a => err(
                vec![],
                match a {
                    MaybeResolvedType::Resolved(ResolvedType::ErrorRecovery) => vec![],
                    _ => vec![CompileError::NotAStruct {
                        name: debug_string.into(),
                        span: debug_span.clone(),
                        actually: a.friendly_type_str(),
                    }],
                },
            ),
        }
    }
}
