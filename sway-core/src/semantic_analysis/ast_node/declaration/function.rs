use crate::{
    error::*,
    parse_tree::*,
    semantic_analysis::{
        ast_node::{
            IsConstant, Mode, TypedCodeBlock, TypedDeclaration, TypedExpression,
            TypedExpressionVariant, TypedReturnStatement, TypedVariableDeclaration,
            VariableMutability,
        },
        create_new_scope, NamespaceWrapper, TypeCheckArguments,
    },
    type_engine::*,
    Ident, TypeParameter,
};

use sway_types::{join_spans, span::Span, Function, Property};

use sha2::{Digest, Sha256};

mod function_parameter;
pub use function_parameter::*;

#[derive(Clone, Debug)]
pub struct TypedFunctionDeclaration {
    pub(crate) name: Ident,
    pub(crate) body: TypedCodeBlock,
    pub(crate) parameters: Vec<TypedFunctionParameter>,
    pub(crate) span: Span,
    pub(crate) return_type: TypeId,
    pub(crate) type_parameters: Vec<TypeParameter>,
    /// Used for error messages -- the span pointing to the return type
    /// annotation of the function
    pub(crate) return_type_span: Span,
    pub(crate) visibility: Visibility,
    /// whether this function exists in another contract and requires a call to it or not
    pub(crate) is_contract_call: bool,
    pub(crate) purity: Purity,
}

impl TypedFunctionDeclaration {
    pub fn type_check(
        arguments: TypeCheckArguments<'_, FunctionDeclaration>,
    ) -> CompileResult<TypedFunctionDeclaration> {
        let TypeCheckArguments {
            checkee: fn_decl,
            namespace,
            crate_namespace,
            self_type,
            build_config,
            dead_code_graph,
            mode,
            mut opts,
            ..
        } = arguments;
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let FunctionDeclaration {
            name,
            body,
            parameters,
            span,
            return_type,
            type_parameters,
            return_type_span,
            visibility,
            purity,
            ..
        } = fn_decl.clone();
        opts.purity = purity;
        // insert type parameters as Unknown types
        let type_mapping = insert_type_parameters(&type_parameters);
        let return_type =
            if let Some(matching_id) = return_type.matches_type_parameter(&type_mapping) {
                insert_type(TypeInfo::Ref(matching_id))
            } else {
                namespace
                    .resolve_type_with_self(return_type, self_type)
                    .unwrap_or_else(|_| {
                        errors.push(CompileError::UnknownType {
                            span: return_type_span.clone(),
                        });
                        insert_type(TypeInfo::ErrorRecovery)
                    })
            };

        // insert parameters and generic type declarations into namespace
        let namespace = create_new_scope(namespace);
        type_parameters.iter().for_each(|param| {
            namespace.insert(param.name_ident.clone(), param.into());
        });
        for FunctionParameter {
            name,
            r#type,
            type_span,
        } in parameters.clone()
        {
            let r#type = if let Some(matching_id) = r#type.matches_type_parameter(&type_mapping) {
                insert_type(TypeInfo::Ref(matching_id))
            } else {
                namespace
                    .resolve_type_with_self(r#type, self_type)
                    .unwrap_or_else(|_| {
                        errors.push(CompileError::UnknownType {
                            span: type_span.clone(),
                        });
                        insert_type(TypeInfo::ErrorRecovery)
                    })
            };
            namespace.insert(
                name.clone(),
                TypedDeclaration::VariableDeclaration(TypedVariableDeclaration {
                    name: name.clone(),
                    body: TypedExpression {
                        expression: TypedExpressionVariant::FunctionParameter,
                        return_type: r#type,
                        is_constant: IsConstant::No,
                        span: name.span().clone(),
                    },
                    is_mutable: VariableMutability::Immutable,
                    const_decl_origin: false,
                    type_ascription: r#type,
                }),
            );
        }

        // If there are no implicit block returns, then we do not want to type check them, so we
        // stifle the errors. If there _are_ implicit block returns, we want to type_check them.
        let (body, _implicit_block_return) = check!(
            TypedCodeBlock::type_check(TypeCheckArguments {
                checkee: body.clone(),
                namespace,
                crate_namespace,
                return_type_annotation: return_type,
                help_text:
                    "Function body's return type does not match up with its return type annotation.",
                self_type,
                build_config,
                dead_code_graph,
                mode: Mode::NonAbi,
                opts,
            }),
            (
                TypedCodeBlock {
                    contents: vec![],
                    whole_block_span: body.whole_block_span,
                },
                crate::type_engine::insert_type(TypeInfo::ErrorRecovery)
            ),
            warnings,
            errors
        );

        let parameters = parameters
            .into_iter()
            .map(
                |FunctionParameter {
                     name,
                     r#type,
                     type_span,
                 }| TypedFunctionParameter {
                    name,
                    r#type: if let Some(matching_id) = r#type.matches_type_parameter(&type_mapping)
                    {
                        insert_type(TypeInfo::Ref(matching_id))
                    } else {
                        namespace
                            .resolve_type_with_self(r#type, self_type)
                            .unwrap_or_else(|_| {
                                errors.push(CompileError::UnknownType {
                                    span: type_span.clone(),
                                });
                                insert_type(TypeInfo::ErrorRecovery)
                            })
                    },
                    type_span,
                },
            )
            .collect::<Vec<_>>();
        // handle the return statement(s)
        let return_statements: Vec<(&TypedExpression, &Span)> = body
            .contents
            .iter()
            .filter_map(|x| {
                if let crate::semantic_analysis::TypedAstNode {
                    content:
                        crate::semantic_analysis::TypedAstNodeContent::ReturnStatement(
                            TypedReturnStatement { ref expr },
                        ),
                    span,
                } = x
                {
                    Some((expr, span))
                } else {
                    None
                }
            })
            .collect();
        for (stmt, span) in return_statements {
            match crate::type_engine::unify_with_self(
                stmt.return_type,
                return_type,
                self_type,
                span,
            ) {
                Ok(mut ws) => {
                    warnings.append(&mut ws);
                }
                Err(e) => {
                    errors.push(CompileError::TypeError(e));
                } //    "Function body's return type does not match up with its return type annotation.",
            }
        }

        // if this is an abi function, it is required that it begins with
        // the three parameters related to contract calls
        //  gas_to_forward: u64,
        //  coins_to_forward: u64,
        //  color_of_coins: b256,
        //
        //  eventually this will be a `ContractRequest`
        //
        //  not spending _too_ much time on particularly specific error messages here since
        //  it is a temporary workaround
        if mode == Mode::ImplAbiFn {
            if parameters.len() == 4 {
                if look_up_type_id(parameters[0].r#type)
                    != TypeInfo::UnsignedInteger(IntegerBits::SixtyFour)
                {
                    errors.push(CompileError::AbiFunctionRequiresSpecificSignature {
                        span: parameters[0].type_span.clone(),
                    });
                }
                if look_up_type_id(parameters[1].r#type)
                    != TypeInfo::UnsignedInteger(IntegerBits::SixtyFour)
                {
                    errors.push(CompileError::AbiFunctionRequiresSpecificSignature {
                        span: parameters[1].type_span.clone(),
                    });
                }
                if look_up_type_id(parameters[2].r#type) != TypeInfo::B256 {
                    errors.push(CompileError::AbiFunctionRequiresSpecificSignature {
                        span: parameters[2].type_span.clone(),
                    });
                }
            } else {
                errors.push(CompileError::AbiFunctionRequiresSpecificSignature {
                    span: parameters
                        .get(0)
                        .map(|x| x.type_span.clone())
                        .unwrap_or_else(|| fn_decl.name.span().clone()),
                });
            }
        }

        ok(
            TypedFunctionDeclaration {
                name,
                body,
                parameters,
                span,
                return_type,
                type_parameters,
                return_type_span,
                visibility,
                // if this is for a contract, then it is a contract call
                is_contract_call: mode == Mode::ImplAbiFn,
                purity,
            },
            warnings,
            errors,
        )
    }
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        self.body.copy_types(type_mapping);
        self.parameters
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping));

        self.return_type = if let Some(matching_id) =
            look_up_type_id(self.return_type).matches_type_parameter(type_mapping)
        {
            insert_type(TypeInfo::Ref(matching_id))
        } else {
            insert_type(look_up_type_id_raw(self.return_type))
        };
    }
    /// Given a typed function declaration with type parameters, make a copy of it and update the
    /// type ids which refer to generic types to be fresh copies, maintaining their referential
    /// relationship. This is used so when this function is resolved, the types don't clobber the
    /// generic type info.
    pub(crate) fn monomorphize(
        &self,
        type_arguments: Vec<(TypeInfo, Span)>,
        self_type: TypeId,
    ) -> CompileResult<TypedFunctionDeclaration> {
        let mut warnings: Vec<CompileWarning> = vec![];
        let mut errors: Vec<CompileError> = vec![];
        debug_assert!(
            !self.type_parameters.is_empty(),
            "Only generic functions can be monomorphized"
        );

        let type_mapping = insert_type_parameters(&self.type_parameters);
        if !type_arguments.is_empty() {
            // check type arguments against parameters
            if self.type_parameters.len() != type_arguments.len() {
                todo!("incorrect number of type args err");
            }

            // check the type arguments
            for ((_, decl_param), (type_argument, type_argument_span)) in
                type_mapping.iter().zip(type_arguments.iter())
            {
                match unify_with_self(
                    *decl_param,
                    insert_type(type_argument.clone()),
                    self_type,
                    type_argument_span,
                ) {
                    Ok(mut ws) => {
                        warnings.append(&mut ws);
                    }
                    Err(e) => {
                        errors.push(e.into());
                        continue;
                    }
                }
            }
        }

        let mut new_decl = self.clone();

        // make all type ids fresh ones
        new_decl
            .body
            .contents
            .iter_mut()
            .for_each(|x| x.copy_types(&type_mapping));

        new_decl
            .parameters
            .iter_mut()
            .for_each(|x| x.copy_types(&type_mapping));

        new_decl.return_type = if let Some(matching_id) =
            look_up_type_id(new_decl.return_type).matches_type_parameter(&type_mapping)
        {
            insert_type(TypeInfo::Ref(matching_id))
        } else {
            insert_type(look_up_type_id_raw(new_decl.return_type))
        };

        ok(new_decl, warnings, errors)
    }
    /// If there are parameters, join their spans. Otherwise, use the fn name span.
    pub(crate) fn parameters_span(&self) -> Span {
        if !self.parameters.is_empty() {
            self.parameters.iter().fold(
                self.parameters[0].name.span().clone(),
                |acc, TypedFunctionParameter { type_span, .. }| join_spans(acc, type_span.clone()),
            )
        } else {
            self.name.span().clone()
        }
    }
    pub(crate) fn replace_self_types(self, self_type: TypeId) -> Self {
        TypedFunctionDeclaration {
            parameters: self
                .parameters
                .iter()
                .map(|x| {
                    let mut x = x.clone();
                    x.r#type = match look_up_type_id(x.r#type) {
                        TypeInfo::SelfType => self_type,
                        _otherwise => x.r#type,
                    };
                    x
                })
                .collect(),
            span: self.span.clone(),
            return_type: match look_up_type_id(self.return_type) {
                TypeInfo::SelfType => self_type,
                _otherwise => self.return_type,
            },
            type_parameters: self.type_parameters.clone(),
            return_type_span: self.return_type_span.clone(),
            ..self
        }
    }
    pub fn to_fn_selector_value_untruncated(&self) -> CompileResult<Vec<u8>> {
        let mut errors = vec![];
        let mut warnings = vec![];
        let mut hasher = Sha256::new();
        let data = check!(
            self.to_selector_name(),
            return err(warnings, errors),
            warnings,
            errors
        );
        hasher.update(data);
        let hash = hasher.finalize();
        ok(hash.to_vec(), warnings, errors)
    }
    /// Converts a [TypedFunctionDeclaration] into a value that is to be used in contract function
    /// selectors.
    /// Hashes the name and parameters using SHA256, and then truncates to four bytes.
    pub fn to_fn_selector_value(&self) -> CompileResult<[u8; 4]> {
        let mut errors = vec![];
        let mut warnings = vec![];
        let hash = check!(
            self.to_fn_selector_value_untruncated(),
            return err(warnings, errors),
            warnings,
            errors
        );
        // 4 bytes truncation via copying into a 4 byte buffer
        let mut buf = [0u8; 4];
        buf.copy_from_slice(&hash[0..4]);
        ok(buf, warnings, errors)
    }

    pub fn to_selector_name(&self) -> CompileResult<String> {
        let mut errors = vec![];
        let mut warnings = vec![];
        let named_params = self
            .parameters
            .iter()
            .map(
                |TypedFunctionParameter {
                     r#type, type_span, ..
                 }| {
                    resolve_type(*r#type, type_span)
                        .expect("unreachable I think?")
                        .to_selector_name(type_span)
                },
            )
            .filter_map(|name| name.ok(&mut warnings, &mut errors))
            .collect::<Vec<String>>();

        ok(
            format!("{}({})", self.name.as_str(), named_params.join(","),),
            warnings,
            errors,
        )
    }

    pub fn generate_json_abi(&self) -> Function {
        Function {
            name: self.name.as_str().to_string(),
            type_field: "function".to_string(),
            inputs: self
                .parameters
                .iter()
                .map(|x| Property {
                    name: x.name.as_str().to_string(),
                    type_field: x.r#type.json_abi_str(),
                    components: x.r#type.generate_json_abi(),
                })
                .collect(),
            outputs: vec![Property {
                name: "".to_string(),
                type_field: self.return_type.json_abi_str(),
                components: self.return_type.generate_json_abi(),
            }],
        }
    }
}

#[test]
fn test_function_selector_behavior() {
    use crate::type_engine::IntegerBits;
    let decl = TypedFunctionDeclaration {
        purity: Default::default(),
        name: Ident::new_with_override(
            "foo",
            Span {
                span: pest::Span::new(" ".into(), 0, 0).unwrap(),
                path: None,
            },
        ),
        body: TypedCodeBlock {
            contents: vec![],
            whole_block_span: Span {
                span: pest::Span::new(" ".into(), 0, 0).unwrap(),
                path: None,
            },
        },
        parameters: vec![],
        span: Span {
            span: pest::Span::new(" ".into(), 0, 0).unwrap(),
            path: None,
        },
        return_type: 0,
        type_parameters: vec![],
        return_type_span: Span {
            span: pest::Span::new(" ".into(), 0, 0).unwrap(),
            path: None,
        },
        visibility: Visibility::Public,
        is_contract_call: false,
    };

    let selector_text = match decl.to_selector_name().value {
        Some(value) => value,
        _ => panic!("test failure"),
    };

    assert_eq!(selector_text, "foo()".to_string());

    let decl = TypedFunctionDeclaration {
        purity: Default::default(),
        name: Ident::new_with_override(
            "bar",
            Span {
                span: pest::Span::new(" ".into(), 0, 0).unwrap(),
                path: None,
            },
        ),
        body: TypedCodeBlock {
            contents: vec![],
            whole_block_span: Span {
                span: pest::Span::new(" ".into(), 0, 0).unwrap(),
                path: None,
            },
        },
        parameters: vec![
            TypedFunctionParameter {
                name: Ident::new_with_override(
                    "foo",
                    Span {
                        span: pest::Span::new(" ".into(), 0, 0).unwrap(),
                        path: None,
                    },
                ),
                r#type: crate::type_engine::insert_type(TypeInfo::Str(5)),
                type_span: Span {
                    span: pest::Span::new(" ".into(), 0, 0).unwrap(),
                    path: None,
                },
            },
            TypedFunctionParameter {
                name: Ident::new_with_override(
                    "baz",
                    Span {
                        span: pest::Span::new(" ".into(), 0, 0).unwrap(),
                        path: None,
                    },
                ),
                r#type: insert_type(TypeInfo::UnsignedInteger(IntegerBits::ThirtyTwo)),
                type_span: Span {
                    span: pest::Span::new(" ".into(), 0, 0).unwrap(),
                    path: None,
                },
            },
        ],
        span: Span {
            span: pest::Span::new(" ".into(), 0, 0).unwrap(),
            path: None,
        },
        return_type: 0,
        type_parameters: vec![],
        return_type_span: Span {
            span: pest::Span::new(" ".into(), 0, 0).unwrap(),
            path: None,
        },
        visibility: Visibility::Public,
        is_contract_call: false,
    };

    let selector_text = match decl.to_selector_name().value {
        Some(value) => value,
        _ => panic!("test failure"),
    };

    assert_eq!(selector_text, "bar(str[5],u32)".to_string());
}
/// Insert all type parameters as unknown types. Return a mapping of type parameter to
/// [TypeId]
pub(crate) fn insert_type_parameters(params: &[TypeParameter]) -> Vec<(TypeParameter, TypeId)> {
    params
        .iter()
        .map(|x| {
            (
                x.clone(),
                insert_type(TypeInfo::UnknownGeneric {
                    name: x.name_ident.clone(),
                }),
            )
        })
        .collect()
}
