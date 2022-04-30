use crate::{
    error::*,
    parse_tree::*,
    semantic_analysis::{
        ast_node::{
            IsConstant, Mode, TypedCodeBlock, TypedDeclaration, TypedExpression,
            TypedExpressionVariant, TypedReturnStatement, TypedVariableDeclaration,
            VariableMutability,
        },
        create_new_scope, NamespaceWrapper, TypeCheckArguments, TypedAstNode, TypedAstNodeContent,
    },
    type_engine::*,
    Ident, TypeParameter,
};

use sway_types::{Function, Property, Span};

use sha2::{Digest, Sha256};

mod function_parameter;
pub use function_parameter::*;

#[derive(Clone, Debug, Eq)]
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

impl From<&TypedFunctionDeclaration> for TypedAstNode {
    fn from(o: &TypedFunctionDeclaration) -> Self {
        let span = o.span.clone();
        TypedAstNode {
            content: TypedAstNodeContent::Declaration(TypedDeclaration::FunctionDeclaration(
                o.clone(),
            )),
            span,
        }
    }
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for TypedFunctionDeclaration {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.body == other.body
            && self.parameters == other.parameters
            && look_up_type_id(self.return_type) == look_up_type_id(other.return_type)
            && self.type_parameters == other.type_parameters
            && self.visibility == other.visibility
            && self.is_contract_call == other.is_contract_call
            && self.purity == other.purity
    }
}

impl TypedFunctionDeclaration {
    pub fn type_check(
        arguments: TypeCheckArguments<'_, FunctionDeclaration>,
    ) -> CompileResult<TypedFunctionDeclaration> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
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
        let FunctionDeclaration {
            name,
            body,
            mut parameters,
            span,
            return_type,
            type_parameters,
            return_type_span,
            visibility,
            purity,
            ..
        } = fn_decl;
        opts.purity = purity;

        // insert type parameters as Unknown types
        let type_mapping = insert_type_parameters(&type_parameters);

        // insert parameters and generic type declarations into namespace
        let namespace = create_new_scope(namespace);

        // check to see if the type parameters shadow one another
        for type_parameter in type_parameters.iter() {
            check!(
                namespace.insert(type_parameter.name_ident.clone(), type_parameter.into()),
                continue,
                warnings,
                errors
            );
        }

        parameters.iter_mut().for_each(|parameter| {
            parameter.type_id =
                match look_up_type_id(parameter.type_id).matches_type_parameter(&type_mapping) {
                    Some(matching_id) => insert_type(TypeInfo::Ref(matching_id)),
                    None => check!(
                        namespace.resolve_type_with_self(
                            look_up_type_id(parameter.type_id),
                            self_type,
                            parameter.type_span.clone(),
                            true
                        ),
                        insert_type(TypeInfo::ErrorRecovery),
                        warnings,
                        errors,
                    ),
                };
        });

        for FunctionParameter { name, type_id, .. } in parameters.clone() {
            namespace.insert(
                name.clone(),
                TypedDeclaration::VariableDeclaration(TypedVariableDeclaration {
                    name: name.clone(),
                    body: TypedExpression {
                        expression: TypedExpressionVariant::FunctionParameter,
                        return_type: type_id,
                        is_constant: IsConstant::No,
                        span: name.span().clone(),
                    },
                    is_mutable: VariableMutability::Immutable,
                    const_decl_origin: false,
                    type_ascription: type_id,
                }),
            );
        }

        let return_type = match return_type.matches_type_parameter(&type_mapping) {
            Some(matching_id) => insert_type(TypeInfo::Ref(matching_id)),
            None => check!(
                namespace.resolve_type_with_self(
                    return_type,
                    self_type,
                    return_type_span.clone(),
                    true
                ),
                insert_type(TypeInfo::ErrorRecovery),
                warnings,
                errors,
            ),
        };

        // If there are no implicit block returns, then we do not want to type check them, so we
        // stifle the errors. If there _are_ implicit block returns, we want to type_check them.
        let (mut body, _implicit_block_return) = check!(
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
                insert_type(TypeInfo::ErrorRecovery)
            ),
            warnings,
            errors
        );
        body.copy_types(&type_mapping);

        let parameters = parameters
            .into_iter()
            .map(
                |FunctionParameter {
                     name,
                     type_id: r#type,
                     type_span,
                 }| TypedFunctionParameter {
                    name,
                    r#type,
                    type_span,
                },
            )
            .collect::<Vec<_>>();
        // handle the return statement(s)
        let return_statements: Vec<&TypedExpression> = body
            .contents
            .iter()
            .flat_map(|node| -> Vec<&TypedReturnStatement> { node.gather_return_statements() })
            .map(|TypedReturnStatement { expr, .. }| expr)
            .collect();
        for stmt in return_statements {
            let (mut new_warnings, new_errors) = unify_with_self(
                stmt.return_type,
                return_type,
                self_type,
                &stmt.span,
                "Return statement must return the declared function return type.",
            );
            warnings.append(&mut new_warnings);
            errors.append(&mut new_errors.into_iter().map(|x| x.into()).collect());
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
        self.type_parameters
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping));

        self.parameters
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping));

        self.return_type =
            match look_up_type_id(self.return_type).matches_type_parameter(type_mapping) {
                Some(matching_id) => insert_type(TypeInfo::Ref(matching_id)),
                None => insert_type(look_up_type_id_raw(self.return_type)),
            };

        self.body.copy_types(type_mapping);
    }

    /// Given a typed function declaration with type parameters, make a copy of it and update the
    /// type ids which refer to generic types to be fresh copies, maintaining their referential
    /// relationship. This is used so when this function is resolved, the types don't clobber the
    /// generic type info.
    pub(crate) fn monomorphize(
        &self,
        type_arguments: Vec<TypeArgument>,
        self_type: TypeId,
    ) -> CompileResult<TypedFunctionDeclaration> {
        let mut warnings: Vec<CompileWarning> = vec![];
        let mut errors: Vec<CompileError> = vec![];
        debug_assert!(
            !self.type_parameters.is_empty(),
            "Only generic functions can be monomorphized"
        );

        let mut new_decl = self.clone();

        let type_mapping = insert_type_parameters(&new_decl.type_parameters);
        if !type_arguments.is_empty() {
            // check type arguments against parameters
            let type_arguments_span = type_arguments
                .iter()
                .map(|x| x.span.clone())
                .reduce(Span::join)
                .unwrap_or_else(|| self.span.clone());
            if new_decl.type_parameters.len() != type_arguments.len() {
                errors.push(CompileError::IncorrectNumberOfTypeArguments {
                    given: type_arguments.len(),
                    expected: new_decl.type_parameters.len(),
                    span: type_arguments_span,
                });
            }

            // check the type arguments
            for ((_, decl_param), type_argument) in type_mapping.iter().zip(type_arguments.iter()) {
                let (mut new_warnings, new_errors) = unify_with_self(
                    *decl_param,
                    type_argument.type_id,
                    self_type,
                    &type_argument.span,
                    "Type argument is not castable to generic type paramter",
                );
                warnings.append(&mut new_warnings);
                errors.append(&mut new_errors.into_iter().map(|x| x.into()).collect());
            }
        }

        // make all type ids fresh ones
        new_decl.copy_types(&type_mapping);
        ok(new_decl, warnings, errors)
    }

    /// If there are parameters, join their spans. Otherwise, use the fn name span.
    pub(crate) fn parameters_span(&self) -> Span {
        if !self.parameters.is_empty() {
            self.parameters.iter().fold(
                self.parameters[0].name.span().clone(),
                |acc, TypedFunctionParameter { type_span, .. }| Span::join(acc, type_span.clone()),
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
        name: Ident::new_no_span("foo"),
        body: TypedCodeBlock {
            contents: vec![],
            whole_block_span: Span::dummy(),
        },
        parameters: vec![],
        span: Span::dummy(),
        return_type: 0,
        type_parameters: vec![],
        return_type_span: Span::dummy(),
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
        name: Ident::new_with_override("bar", Span::dummy()),
        body: TypedCodeBlock {
            contents: vec![],
            whole_block_span: Span::dummy(),
        },
        parameters: vec![
            TypedFunctionParameter {
                name: Ident::new_no_span("foo"),
                r#type: crate::type_engine::insert_type(TypeInfo::Str(5)),
                type_span: Span::dummy(),
            },
            TypedFunctionParameter {
                name: Ident::new_no_span("baz"),
                r#type: insert_type(TypeInfo::UnsignedInteger(IntegerBits::ThirtyTwo)),
                type_span: Span::dummy(),
            },
        ],
        span: Span::dummy(),
        return_type: 0,
        type_parameters: vec![],
        return_type_span: Span::dummy(),
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
pub(crate) fn insert_type_parameters(
    type_parameters: &[TypeParameter],
) -> Vec<(TypeParameter, TypeId)> {
    type_parameters
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
