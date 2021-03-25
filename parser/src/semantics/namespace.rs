use super::{
    ast_node::{TypedExpressionVariant, TypedStructExpressionField, TypedVariableDeclaration},
    TypedExpression,
};
use crate::error::*;
use crate::CallPath;
use crate::{CompileResult, TypeInfo};
use crate::{Ident, TypedDeclaration, TypedFunctionDeclaration};
use std::collections::HashMap;

type ModuleName = String;

#[derive(Clone, Debug, Default)]
pub struct Namespace<'sc> {
    symbols: HashMap<Ident<'sc>, TypedDeclaration<'sc>>,
    implemented_traits: HashMap<(Ident<'sc>, TypeInfo<'sc>), Vec<TypedFunctionDeclaration<'sc>>>,
    /// any imported namespaces associated with an ident which is a  library name
    modules: HashMap<ModuleName, Namespace<'sc>>,
}

impl<'sc> Namespace<'sc> {
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

    pub(crate) fn item_import(
        &mut self,
        path: Vec<Ident<'sc>>,
        item: &Ident<'sc>,
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
    #[allow(dead_code)]
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

    pub(crate) fn find_module(&self, path: &Vec<Ident<'sc>>) -> CompileResult<'sc, Namespace<'sc>> {
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
    pub(crate) fn insert_trait_implementation(
        &mut self,
        trait_name: Ident<'sc>,
        type_implementing_for: TypeInfo<'sc>,
        functions_buf: Vec<TypedFunctionDeclaration<'sc>>,
    ) -> CompileResult<()> {
        let mut warnings = vec![];
        if let Some(_) = self
            .implemented_traits
            .get(&(trait_name.clone(), type_implementing_for.clone()))
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
        self.implemented_traits
            .insert((trait_name, type_implementing_for), functions_buf);
        ok((), warnings, vec![])
    }

    pub fn insert_module(&mut self, module_name: String, module_contents: Namespace<'sc>) {
        self.modules.insert(module_name, module_contents);
    }

    pub(crate) fn find_subfield(
        &self,
        subfield_exp: Vec<Ident<'sc>>,
    ) -> CompileResult<'sc, TypedExpression<'sc>> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let mut ident_iter = subfield_exp.into_iter();
        let first_ident = ident_iter.next().unwrap();
        let symbol = match self.symbols.get(&first_ident) {
            Some(s) => s,
            None => {
                errors.push(CompileError::UnknownVariable {
                    var_name: first_ident.primary_name,
                    span: first_ident.span,
                });
                return err(warnings, errors);
            }
        };
        let (mut fields, mut struct_name) = match get_struct_expression_fields(symbol, &first_ident)
        {
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

        let mut expr = None;

        assert!(ident_iter.clone().count() > 0);
        for ident in ident_iter {
            let TypedStructExpressionField { value, .. } =
                match fields.iter().find(|x| x.name == ident.primary_name) {
                    Some(field) => field.clone(),
                    None => {
                        let field_name = ident.primary_name.clone();
                        let available_fields =
                            fields.iter().map(|x| x.name.clone()).collect::<Vec<_>>();

                        errors.push(CompileError::FieldNotFound {
                            field_name,
                            struct_name: struct_name.primary_name.clone(),
                            available_fields: available_fields.join(", "),
                            span: ident.span,
                        });
                        return err(warnings, errors);
                    }
                };
            match &value {
                TypedExpression {
                    expression:
                        TypedExpressionVariant::StructExpression {
                            fields: l_fields,
                            struct_name: l_struct_name,
                            ..
                        },
                    ..
                } => {
                    fields = l_fields.into_iter().cloned().collect();
                    struct_name = l_struct_name.clone();
                    expr = Some(value);
                }
                _ => {
                    fields = vec![];
                    expr = Some(value);
                }
            }
        }
        // unwrap is safe: note that all branches above assign to expr
        ok(expr.unwrap(), warnings, errors)
    }

    pub(crate) fn get_methods_for_type(
        &self,
        r#type: TypeInfo<'sc>,
    ) -> Option<Vec<TypedFunctionDeclaration<'sc>>> {
        for ((_trait_name, type_info), methods) in &self.implemented_traits {
            if *type_info == r#type {
                return Some(methods.clone());
            }
        }
        None
    }

    pub(crate) fn find_method_for_type(
        &self,
        r#type: TypeInfo<'sc>,
        method_name: Ident<'sc>,
    ) -> Option<TypedFunctionDeclaration<'sc>> {
        let methods = self.get_methods_for_type(r#type)?;
        methods
            .into_iter()
            .find(|TypedFunctionDeclaration { name, .. }| *name == method_name)
    }
}

fn get_struct_expression_fields<'sc>(
    decl: &TypedDeclaration<'sc>,
    debug_ident: &Ident<'sc>,
) -> CompileResult<'sc, (Vec<TypedStructExpressionField<'sc>>, Ident<'sc>)> {
    match decl {
        TypedDeclaration::VariableDeclaration(TypedVariableDeclaration {
            body:
                TypedExpression {
                    expression:
                        TypedExpressionVariant::StructExpression {
                            fields,
                            struct_name,
                            ..
                        },
                    ..
                },
            ..
        }) => ok((fields.clone(), struct_name.clone()), vec![], vec![]),
        TypedDeclaration::VariableDeclaration(TypedVariableDeclaration { .. }) => err(
            vec![],
            vec![CompileError::NotAStruct {
                name: debug_ident.span.as_str(),
                span: debug_ident.span.clone(),
            }],
        ),
        o => todo!(
            "err: {} is not a struct with field {}",
            o.friendly_name(),
            debug_ident.primary_name
        ),
    }
}
