use super::{
    ast_node::{
        TypedEnumDeclaration, TypedStructDeclaration, TypedStructField, TypedVariableDeclaration,
    },
    TypedExpression,
};
use crate::error::*;
use crate::types::{MaybeResolvedType, PartiallyResolvedType, ResolvedType};
use crate::CallPath;
use crate::{CompileResult, TypeInfo};
use crate::{Ident, TypedDeclaration, TypedFunctionDeclaration};
use std::collections::HashMap;

type ModuleName = String;

#[derive(Clone, Debug, Default)]
pub struct Namespace<'sc> {
    symbols: HashMap<Ident<'sc>, TypedDeclaration<'sc>>,
    implemented_traits:
        HashMap<(Ident<'sc>, MaybeResolvedType<'sc>), Vec<TypedFunctionDeclaration<'sc>>>,
    /// any imported namespaces associated with an ident which is a  library name
    modules: HashMap<ModuleName, Namespace<'sc>>,
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
    /// Given a path to a module, import everything from it and merge it into this namespace.
    /// This is used when an import path contains an asterisk.
    pub(crate) fn star_import(&mut self, idents: Vec<Ident<'sc>>) -> CompileResult<()> {
        let idents_buf = idents.into_iter();
        let mut namespace = self.clone();
        for ident in idents_buf {
            match namespace.modules.get(ident.primary_name) {
                Some(o) => namespace = o.clone(),
                None => {
                    return err(
                        vec![],
                        vec![CompileError::ModuleNotFound {
                            span: ident.span,
                            name: ident.primary_name.to_string(),
                        }],
                    )
                }
            };
        }
        self.merge_namespaces(&namespace);
        ok((), vec![], vec![])
    }

    /// Pull a single item from a module and import it into this namespace.
    pub(crate) fn item_import(
        &mut self,
        path: Vec<Ident<'sc>>,
        item: &Ident<'sc>,
        // TODO support aliasing in grammar -- see alias
        alias: Option<Ident<'sc>>,
    ) -> CompileResult<()> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let namespace = type_check!(
            self.find_module(&path),
            return err(warnings, errors),
            warnings,
            errors
        );

        match namespace.symbols.get(item) {
            Some(TypedDeclaration::TraitDeclaration(tr)) => {
                let name = match alias {
                    Some(s) => s.clone(),
                    None => item.clone(),
                };
                // import the trait itself
                self.insert(name.clone(), TypedDeclaration::TraitDeclaration(tr.clone()));

                // find implementations of this trait and import them
                namespace
                    .implemented_traits
                    .iter()
                    .filter(|((trait_name, _ty), _)| item == trait_name)
                    .for_each(|((_trait_name, trait_type), methods)| {
                        self.implemented_traits
                            .insert((name.clone(), trait_type.clone()), methods.clone());
                    });
            }
            Some(o) => {
                let name = match alias {
                    Some(s) => s.clone(),
                    None => item.clone(),
                };
                self.insert(name, o.clone());
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
    }

    pub(crate) fn insert(
        &mut self,
        name: Ident<'sc>,
        item: TypedDeclaration<'sc>,
    ) -> CompileResult<()> {
        let mut warnings = vec![];
        if let Some(_) = self.symbols.get(&name) {
            warnings.push(CompileWarning {
                span: name.span.clone(),
                warning_content: Warning::OverridesOtherSymbol {
                    name: name.span.clone().as_str(),
                },
            });
        }
        self.symbols.insert(name, item.clone());
        ok((), warnings, vec![])
    }

    pub(crate) fn get_symbol(&self, symbol: &Ident<'sc>) -> Option<&TypedDeclaration<'sc>> {
        self.symbols.get(symbol)
    }

    /// Used for calls that look like this:
    /// `foo::bar::function`
    /// where `foo` and `bar` are the prefixes
    /// and `function` is the suffix
    pub(crate) fn get_call_path(
        &self,
        path: &CallPath<'sc>,
    ) -> CompileResult<'sc, TypedDeclaration<'sc>> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let module = type_check!(
            self.find_module(&path.prefixes),
            return err(warnings, errors),
            warnings,
            errors
        );

        match module.symbols.get(&path.suffix).cloned() {
            Some(o) => ok(o, warnings, errors),
            None => {
                errors.push(CompileError::SymbolNotFound {
                    name: path.suffix.primary_name,
                    span: path.suffix.span.clone(),
                });
                err(warnings, errors)
            }
        }
    }

    pub(crate) fn find_module(&self, path: &[Ident<'sc>]) -> CompileResult<'sc, Namespace<'sc>> {
        let mut namespace = self.clone();
        let mut errors = vec![];
        let warnings = vec![];
        for ident in path {
            match namespace.modules.get(ident.primary_name) {
                Some(o) => namespace = o.clone(),
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
    pub(crate) fn find_module_mut(
        &mut self,
        path: &[Ident<'sc>],
    ) -> CompileResult<'sc, &mut Namespace<'sc>> {
        let mut namespace = self;
        let mut errors = vec![];
        let warnings = vec![];
        for ident in path {
            match namespace.modules.get_mut(ident.primary_name) {
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
        let mut errors = vec![];
        let module_to_insert_into = type_check!(
            self.find_module_mut(&trait_name.prefixes),
            return err(warnings, errors),
            warnings,
            errors
        );
        if let Some(_) = module_to_insert_into
            .implemented_traits
            .get(&(trait_name.suffix.clone(), type_implementing_for.clone()))
        {
            warnings.push(CompileWarning {
                warning_content: Warning::OverridingTraitImplementation,
                span: functions_buf.iter().fold(
                    functions_buf[0].span.clone(),
                    |acc, TypedFunctionDeclaration { span, .. }| {
                        crate::utils::join_spans(acc, span.clone())
                    },
                ),
            })
        }
        module_to_insert_into
            .implemented_traits
            .insert((trait_name.suffix, type_implementing_for), functions_buf);
        ok((), warnings, errors)
    }

    pub fn insert_module(&mut self, module_name: String, module_contents: Namespace<'sc>) {
        self.modules.insert(module_name, module_contents);
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
        let mut ident_iter = subfield_exp.into_iter().peekable();
        let first_ident = ident_iter.next().unwrap();
        let symbol = match self.symbols.get(&first_ident) {
            Some(s) => s,
            None => {
                errors.push(CompileError::UnknownVariable {
                    var_name: first_ident.primary_name,
                    span: first_ident.span.clone(),
                });
                return err(warnings, errors);
            }
        };
        if ident_iter.peek().is_none() {
            let ty = type_check!(
                symbol.return_type(),
                return err(warnings, errors),
                warnings,
                errors
            );
            return ok((ty.clone(), ty), warnings, errors);
        }
        let (mut fields, struct_name) = match self.get_struct_type_fields(symbol, &first_ident) {
            CompileResult::Ok {
                value,
                warnings: mut l_w,
                errors: mut l_e,
            } => {
                errors.append(&mut l_e);
                warnings.append(&mut l_w);
                value
            }
            CompileResult::Err {
                warnings: mut l_w,
                errors: mut l_e,
            } => {
                errors.append(&mut l_e);
                warnings.append(&mut l_w);
                // if it is missing, the error message comes from within the above method
                // so we don't need to re-add it here
                return err(warnings, errors);
            }
        };

        let mut ret_ty = type_check!(
            symbol.return_type(),
            return err(warnings, errors),
            warnings,
            errors
        );
        let mut parent_rover = ret_ty.clone();

        for ident in ident_iter {
            // find the ident in the currently available fields
            let TypedStructField { r#type, .. } = match fields.iter().find(|x| x.name == *ident) {
                Some(field) => field.clone(),
                None => {
                    // gather available fields for the error message
                    let field_name = ident.primary_name.clone();
                    let available_fields = fields
                        .iter()
                        .map(|x| x.name.primary_name.clone())
                        .collect::<Vec<_>>();

                    errors.push(CompileError::FieldNotFound {
                        field_name,
                        struct_name: struct_name.primary_name.clone(),
                        available_fields: available_fields.join(", "),
                        span: ident.span.clone(),
                    });
                    return err(warnings, errors);
                }
            };
            match r#type {
                ResolvedType::Struct { .. } => {
                    let (l_fields, _l_name) = type_check!(
                        self.find_struct_name_and_fields(
                            &MaybeResolvedType::Resolved(r#type),
                            &ident
                        ),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    parent_rover = ret_ty.clone();
                    fields = l_fields;
                }
                _ => {
                    fields = vec![];
                    parent_rover = ret_ty.clone();
                    ret_ty = MaybeResolvedType::Resolved(r#type);
                }
            }
        }
        ok((ret_ty, parent_rover), warnings, errors)
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

    pub(crate) fn find_method_for_type(
        &self,
        r#type: &MaybeResolvedType<'sc>,
        method_name: Ident<'sc>,
    ) -> Option<TypedFunctionDeclaration<'sc>> {
        let methods = self.get_methods_for_type(r#type);
        methods
            .into_iter()
            .find(|TypedFunctionDeclaration { name, .. }| *name == method_name)
    }
    /// given a declaration that may refer to a variable which contains a struct,
    /// find that struct's fields and name for use in determining if a subfield expression is valid
    /// e.g. foo.bar.baz
    /// is foo a struct? does it contain a field bar? is foo.bar a struct? does foo.bar contain a
    /// field baz? this is the problem this function addresses
    fn get_struct_type_fields(
        &self,
        decl: &TypedDeclaration<'sc>,
        debug_ident: &Ident<'sc>,
    ) -> CompileResult<'sc, (Vec<TypedStructField<'sc>>, &Ident<'sc>)> {
        match decl {
            TypedDeclaration::VariableDeclaration(TypedVariableDeclaration {
                body: TypedExpression { return_type, .. },
                ..
            }) => self.find_struct_name_and_fields(return_type, debug_ident),
            a => {
                return err(
                    vec![],
                    vec![CompileError::NotAStruct {
                        name: debug_ident.primary_name.clone(),
                        span: debug_ident.span.clone(),
                        actually: a.friendly_name().to_string(),
                    }],
                )
            }
        }
    }
    /// given a type, look that type up in the namespace and:
    /// 1) assert that it is a struct, return error otherwise
    /// 2) return its fields and struct name
    fn find_struct_name_and_fields(
        &self,
        return_type: &MaybeResolvedType<'sc>,
        debug_ident: &Ident<'sc>,
    ) -> CompileResult<'sc, (Vec<TypedStructField<'sc>>, &Ident<'sc>)> {
        if let MaybeResolvedType::Resolved(ResolvedType::Struct { name, fields: _ }) = return_type {
            match self.get_symbol(name) {
                Some(TypedDeclaration::StructDeclaration(TypedStructDeclaration {
                    fields,
                    name,
                    ..
                })) => ok((fields.clone(), name), vec![], vec![]),
                Some(a) => err(
                    vec![],
                    vec![CompileError::NotAStruct {
                        name: debug_ident.span.as_str(),
                        span: debug_ident.span.clone(),
                        actually: a.friendly_name().to_string(),
                    }],
                ),
                None => err(
                    vec![],
                    vec![CompileError::SymbolNotFound {
                        name: debug_ident.span.as_str(),
                        span: debug_ident.span.clone(),
                    }],
                ),
            }
        } else {
            err(
                vec![],
                vec![CompileError::NotAStruct {
                    name: debug_ident.span.as_str(),
                    span: debug_ident.span.clone(),
                    actually: return_type.friendly_type_str(),
                }],
            )
        }
    }
}
