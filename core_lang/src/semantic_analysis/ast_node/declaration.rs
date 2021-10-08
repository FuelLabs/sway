use super::impl_trait::Mode;
use super::{
    IsConstant, TypedCodeBlock, TypedExpression, TypedExpressionVariant, TypedReturnStatement,
};
use crate::asm_generation::AsmNamespace;
use crate::parse_tree::*;
use crate::semantic_analysis::Namespace;
use crate::span::Span;
use crate::type_engine::TypeId;
use crate::{
    build_config::BuildConfig,
    error::*,
    types::{IntegerBits, MaybeResolvedType, PartiallyResolvedType, ResolvedType},
    Ident,
};
use crate::{control_flow_analysis::ControlFlowGraph, types::TypeInfo};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug)]
pub enum TypedDeclaration<'sc> {
    VariableDeclaration(TypedVariableDeclaration<'sc>),
    ConstantDeclaration(TypedConstantDeclaration<'sc>),
    FunctionDeclaration(TypedFunctionDeclaration<'sc>),
    TraitDeclaration(TypedTraitDeclaration<'sc>),
    StructDeclaration(TypedStructDeclaration<'sc>),
    EnumDeclaration(TypedEnumDeclaration<'sc>),
    Reassignment(TypedReassignment<'sc>),
    ImplTrait {
        trait_name: CallPath<'sc>,
        span: Span<'sc>,
        methods: Vec<TypedFunctionDeclaration<'sc>>,
        type_implementing_for: MaybeResolvedType<'sc>,
    },
    AbiDeclaration(TypedAbiDeclaration<'sc>),
    // no contents since it is a side-effectful declaration, i.e it populates a namespace
    SideEffect,
    ErrorRecovery,
}

impl<'sc> TypedDeclaration<'sc> {
    /// friendly name string used for error reporting.
    pub(crate) fn friendly_name(&self) -> &'static str {
        use TypedDeclaration::*;
        match self {
            VariableDeclaration(_) => "variable",
            ConstantDeclaration(_) => "constant",
            FunctionDeclaration(_) => "function",
            TraitDeclaration(_) => "trait",
            StructDeclaration(_) => "struct",
            EnumDeclaration(_) => "enum",
            Reassignment(_) => "reassignment",
            ImplTrait { .. } => "impl trait",
            AbiDeclaration(..) => "abi",
            SideEffect => "",
            ErrorRecovery => "error",
        }
    }
    pub(crate) fn return_type(&self, namespace: &mut Namespace<'sc>) -> CompileResult<'sc, TypeId> {
        ok(
            match self {
                TypedDeclaration::VariableDeclaration(TypedVariableDeclaration {
                    body, ..
                }) => body.return_type.clone(),
                TypedDeclaration::FunctionDeclaration { .. } => {
                    return err(
                        vec![],
                        vec![CompileError::Unimplemented(
                            "Function pointers have not yet been implemented.",
                            self.span(),
                        )],
                    )
                }
                TypedDeclaration::StructDeclaration(TypedStructDeclaration {
                    name,
                    fields,
                    ..
                }) => namespace.insert_type(TypeInfo::Struct {
                    name: name.clone(),
                    fields: fields.clone(),
                }),
                TypedDeclaration::Reassignment(TypedReassignment { rhs, .. }) => {
                    rhs.return_type.clone()
                }
                decl => {
                    return err(
                        vec![],
                        vec![CompileError::NotAType {
                            span: decl.span(),
                            name: decl.pretty_print(),
                            actually_is: decl.friendly_name(),
                        }],
                    )
                }
            },
            vec![],
            vec![],
        )
    }

    pub(crate) fn span(&self) -> Span<'sc> {
        use TypedDeclaration::*;
        match self {
            VariableDeclaration(TypedVariableDeclaration { name, .. }) => name.span.clone(),
            ConstantDeclaration(TypedConstantDeclaration { name, .. }) => name.span.clone(),
            FunctionDeclaration(TypedFunctionDeclaration { span, .. }) => span.clone(),
            TraitDeclaration(TypedTraitDeclaration { name, .. }) => name.span.clone(),
            StructDeclaration(TypedStructDeclaration { name, .. }) => name.span.clone(),
            EnumDeclaration(TypedEnumDeclaration { span, .. }) => span.clone(),
            Reassignment(TypedReassignment { lhs, .. }) => {
                lhs.iter().fold(lhs[0].span(), |acc, this| {
                    crate::utils::join_spans(acc, this.span())
                })
            }
            AbiDeclaration(TypedAbiDeclaration { span, .. }) => span.clone(),
            ImplTrait { span, .. } => span.clone(),
            SideEffect | ErrorRecovery => unreachable!("No span exists for these ast node types"),
        }
    }

    pub(crate) fn pretty_print(&self) -> String {
        format!(
            "{} declaration ({})",
            self.friendly_name(),
            match self {
                TypedDeclaration::VariableDeclaration(TypedVariableDeclaration {
                    is_mutable,
                    name,
                    ..
                }) => format!(
                    "{} {}",
                    if *is_mutable { "mut" } else { "" },
                    name.primary_name
                ),
                TypedDeclaration::FunctionDeclaration(TypedFunctionDeclaration {
                    name, ..
                }) => {
                    name.primary_name.into()
                }
                TypedDeclaration::TraitDeclaration(TypedTraitDeclaration { name, .. }) =>
                    name.primary_name.into(),
                TypedDeclaration::StructDeclaration(TypedStructDeclaration { name, .. }) =>
                    name.primary_name.into(),
                TypedDeclaration::EnumDeclaration(TypedEnumDeclaration { name, .. }) =>
                    name.primary_name.into(),
                TypedDeclaration::Reassignment(TypedReassignment { lhs, .. }) => lhs
                    .iter()
                    .map(|x| x.name.primary_name)
                    .collect::<Vec<_>>()
                    .join("."),
                _ => String::new(),
            }
        )
    }
}

/// A `TypedAbiDeclaration` contains the type-checked version of the parse tree's [AbiDeclaration].
#[derive(Clone, Debug)]
pub struct TypedAbiDeclaration<'sc> {
    /// The name of the abi trait (also known as a "contract trait")
    pub(crate) name: Ident<'sc>,
    /// The methods a contract is required to implement in order opt in to this interface
    pub(crate) interface_surface: Vec<TypedTraitFn<'sc>>,
    /// The methods provided to a contract "for free" upon opting in to this interface
    pub(crate) methods: Vec<FunctionDeclaration<'sc>>,
    pub(crate) span: Span<'sc>,
}

#[derive(Clone, Debug)]
pub struct TypedStructDeclaration<'sc> {
    pub(crate) name: Ident<'sc>,
    pub(crate) fields: Vec<TypedStructField<'sc>>,
    pub(crate) type_parameters: Vec<TypeParameter<'sc>>,
    pub(crate) visibility: Visibility,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TypedStructField<'sc> {
    pub(crate) name: Ident<'sc>,
    pub(crate) r#type: TypeId,
    pub(crate) span: Span<'sc>,
}

#[derive(Clone, Debug)]
pub struct TypedEnumDeclaration<'sc> {
    pub(crate) name: Ident<'sc>,
    pub(crate) type_parameters: Vec<TypeParameter<'sc>>,
    pub(crate) variants: Vec<TypedEnumVariant<'sc>>,
    pub(crate) span: Span<'sc>,
}
impl<'sc> TypedEnumDeclaration<'sc> {
    /// Given type arguments, match them up with the type parameters and return the result.
    /// Currently unimplemented as we don't support generic enums yet, but when we do, this will be
    /// the place to resolve those typed.
    pub(crate) fn resolve_generic_types(
        &self,
        _type_arguments: Vec<MaybeResolvedType<'sc>>,
    ) -> CompileResult<'sc, Self> {
        ok(self.clone(), vec![], vec![])
    }
    /// Returns the [ResolvedType] corresponding to this enum's type.
    pub(crate) fn as_type(&self, namespace: &mut Namespace<'sc>) -> TypeId {
        namespace.insert_type(TypeInfo::Enum {
            name: self.name.clone(),
            variant_types: self.variants.iter().map(|x| x.r#type.clone()).collect(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct TypedEnumVariant<'sc> {
    pub(crate) name: Ident<'sc>,
    pub(crate) r#type: TypeId,
    pub(crate) tag: usize,
    pub(crate) span: Span<'sc>,
}

#[derive(Clone, Debug)]
pub struct TypedVariableDeclaration<'sc> {
    pub(crate) name: Ident<'sc>,
    pub(crate) body: TypedExpression<'sc>, // will be codeblock variant
    pub(crate) is_mutable: bool,
}

#[derive(Clone, Debug)]
pub struct TypedConstantDeclaration<'sc> {
    pub(crate) name: Ident<'sc>,
    pub(crate) value: TypedExpression<'sc>,
}

// TODO: type check generic type args and their usage
#[derive(Clone, Debug)]
pub struct TypedFunctionDeclaration<'sc> {
    pub(crate) name: Ident<'sc>,
    pub(crate) body: TypedCodeBlock<'sc>,
    pub(crate) parameters: Vec<TypedFunctionParameter<'sc>>,
    pub(crate) span: Span<'sc>,
    pub(crate) return_type: TypeId,
    pub(crate) type_parameters: Vec<TypeParameter<'sc>>,
    /// Used for error messages -- the span pointing to the return type
    /// annotation of the function
    pub(crate) return_type_span: Span<'sc>,
    pub(crate) visibility: Visibility,
    /// whether this function exists in another contract and requires a call to it or not
    pub(crate) is_contract_call: bool,
}

impl<'sc> TypedFunctionDeclaration<'sc> {
    /// If there are parameters, join their spans. Otherwise, use the fn name span.
    pub(crate) fn parameters_span(&self) -> Span<'sc> {
        if self.parameters.len() >= 1 {
            self.parameters.iter().fold(
                self.parameters[0].name.span.clone(),
                |acc, TypedFunctionParameter { type_span, .. }| {
                    crate::utils::join_spans(acc, type_span.clone())
                },
            )
        } else {
            self.name.span.clone()
        }
    }
    pub(crate) fn replace_self_types(&self, self_type: TypeId) -> Self {
        todo!()
        // TypedFunctionDeclaration {
        //     name: self.name.clone(),
        //     body: self.body.replace_self_types(self_type),
        //     parameters: self
        //         .parameters
        //         .iter()
        //         .map(|x| {
        //             let mut x = x.clone();
        //             x.r#type = match x.r#type {
        //                 MaybeResolvedType::Partial(PartiallyResolvedType::SelfType) => {
        //                     self_type.clone()
        //                 }
        //                 otherwise => otherwise.clone(),
        //             };
        //             x
        //         })
        //         .collect(),
        //     span: self.span.clone(),
        //     return_type: match &self.return_type {
        //         MaybeResolvedType::Partial(PartiallyResolvedType::SelfType) => self_type.clone(),
        //         otherwise => otherwise.clone(),
        //     },
        //     type_parameters: self.type_parameters.clone(),
        //     return_type_span: self.return_type_span.clone(),
        //     visibility: self.visibility.clone(),
        //     is_contract_call: self.is_contract_call,
        // }
    }
    pub fn to_fn_selector_value_untruncated(&self) -> CompileResult<'sc, Vec<u8>> {
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
    pub fn to_fn_selector_value(&self) -> CompileResult<'sc, [u8; 4]> {
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

    pub fn to_selector_name(&self, namespace: &AsmNamespace<'sc>) -> CompileResult<'sc, String> {
        let mut errors = vec![];
        let mut warnings = vec![];
        let named_params = self
            .parameters
            .iter()
            .map(
                |TypedFunctionParameter {
                     r#type, type_span, ..
                 }| {
                    namespace
                        .resolve_type(*r#type, type_span)
                        .expect("unreachable I think?")
                        .to_selector_name(type_span)
                },
            )
            .filter_map(|name| name.ok(&mut warnings, &mut errors))
            .collect::<Vec<String>>();

        ok(
            format!("{}({})", self.name.primary_name, named_params.join(","),),
            warnings,
            errors,
        )
    }
}

#[test]
fn test_function_selector_behavior() {
    use crate::types::IntegerBits;
    let decl = TypedFunctionDeclaration {
        name: Ident {
            primary_name: "foo",
            span: Span {
                span: pest::Span::new(" ", 0, 0).unwrap(),
                path: None,
            },
        },
        body: TypedCodeBlock {
            contents: vec![],
            whole_block_span: Span {
                span: pest::Span::new(" ", 0, 0).unwrap(),
                path: None,
            },
        },
        parameters: vec![],
        span: Span {
            span: pest::Span::new(" ", 0, 0).unwrap(),
            path: None,
        },
        return_type: 0,
        type_parameters: vec![],
        return_type_span: Span {
            span: pest::Span::new(" ", 0, 0).unwrap(),
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
        name: Ident {
            primary_name: "bar",
            span: Span {
                span: pest::Span::new(" ", 0, 0).unwrap(),
                path: None,
            },
        },
        body: TypedCodeBlock {
            contents: vec![],
            whole_block_span: Span {
                span: pest::Span::new(" ", 0, 0).unwrap(),
                path: None,
            },
        },
        parameters: vec![
            TypedFunctionParameter {
                name: Ident {
                    primary_name: "foo",
                    span: Span {
                        span: pest::Span::new(" ", 0, 0).unwrap(),
                        path: None,
                    },
                },
                r#type: todo!("Type id for MaybeResolvedType::Resolved(ResolvedType::Str(5))"),
                type_span: Span {
                    span: pest::Span::new(" ", 0, 0).unwrap(),
                    path: None,
                },
            },
            TypedFunctionParameter {
                name: Ident {
                    primary_name: "baz",
                    span: Span {
                        span: pest::Span::new(" ", 0, 0).unwrap(),
                        path: None,
                    },
                },
                r#type: todo!(
                    "type id for MaybeResolvedType::Resolved(ResolvedType::UnsignedInteger(
                    IntegerBits::ThirtyTwo,
                ))"
                ),
                type_span: Span {
                    span: pest::Span::new(" ", 0, 0).unwrap(),
                    path: None,
                },
            },
        ],
        span: Span {
            span: pest::Span::new(" ", 0, 0).unwrap(),
            path: None,
        },
        return_type: 0,
        type_parameters: vec![],
        return_type_span: Span {
            span: pest::Span::new(" ", 0, 0).unwrap(),
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedFunctionParameter<'sc> {
    pub(crate) name: Ident<'sc>,
    pub(crate) r#type: TypeId,
    pub(crate) type_span: Span<'sc>,
}

#[derive(Clone, Debug)]
pub struct TypedTraitDeclaration<'sc> {
    pub(crate) name: Ident<'sc>,
    pub(crate) interface_surface: Vec<TypedTraitFn<'sc>>,
    pub(crate) methods: Vec<FunctionDeclaration<'sc>>,
    pub(crate) type_parameters: Vec<TypeParameter<'sc>>,
    pub(crate) visibility: Visibility,
}
#[derive(Clone, Debug)]
pub struct TypedTraitFn<'sc> {
    pub(crate) name: Ident<'sc>,
    pub(crate) parameters: Vec<TypedFunctionParameter<'sc>>,
    pub(crate) return_type: TypeId,
    pub(crate) return_type_span: Span<'sc>,
}

/// Represents the left hand side of a reassignment -- a name to locate it in the
/// namespace, and the type that the name refers to. The type is used for memory layout
/// in asm generation.
#[derive(Clone, Debug)]
pub struct ReassignmentLhs<'sc> {
    pub(crate) name: Ident<'sc>,
    pub(crate) r#type: TypeId,
}

impl<'sc> ReassignmentLhs<'sc> {
    pub(crate) fn span(&self) -> Span<'sc> {
        self.name.span.clone()
    }
}

#[derive(Clone, Debug)]
pub struct TypedReassignment<'sc> {
    // either a direct variable, so length of 1, or
    // at series of struct fields/array indices (array syntax)
    pub(crate) lhs: Vec<ReassignmentLhs<'sc>>,
    pub(crate) rhs: TypedExpression<'sc>,
}

impl<'sc> TypedFunctionDeclaration<'sc> {
    pub fn type_check(
        fn_decl: FunctionDeclaration<'sc>,
        namespace: &mut Namespace<'sc>,
        _return_type_annotation: TypeId,
        _help_text: impl Into<String>,
        // If there are any `Self` types in this declaration,
        // resolve them to this type.
        self_type: TypeId,
        build_config: &BuildConfig,
        dead_code_graph: &mut ControlFlowGraph<'sc>,
        mode: Mode,
    ) -> CompileResult<'sc, TypedFunctionDeclaration<'sc>> {
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
            ..
        } = fn_decl.clone();
        let return_type = namespace.resolve_type(return_type, self_type);
        // insert parameters into namespace
        let mut namespace = namespace.clone();
        for FunctionParameter { name, r#type, .. } in parameters.clone() {
            let r#type = namespace.resolve_type(r#type, self_type);
            namespace.insert(
                name.clone(),
                TypedDeclaration::VariableDeclaration(TypedVariableDeclaration {
                    name: name.clone(),
                    body: TypedExpression {
                        expression: TypedExpressionVariant::FunctionParameter,
                        return_type: r#type,
                        is_constant: IsConstant::No,
                        span: name.span.clone(),
                    },
                    is_mutable: false, // TODO allow mutable function params?
                }),
            );
        }

        // If there are no implicit block returns, then we do not want to type check them, so we
        // stifle the errors. If there _are_ implicit block returns, we want to type_check them.
        let (body, _implicit_block_return) = check!(
            TypedCodeBlock::type_check(
                body.clone(),
                &namespace,
                return_type.clone(),
                "Function body's return type does not match up with its return type annotation.",
                self_type,
                build_config,
                dead_code_graph
            ),
            (
                TypedCodeBlock {
                    contents: vec![],
                    whole_block_span: body.whole_block_span.clone()
                },
                Some(MaybeResolvedType::Resolved(ResolvedType::ErrorRecovery))
            ),
            warnings,
            errors
        );

        // check the generic types in the arguments, make sure they are in the type
        // scope
        let parameters = parameters
            .into_iter()
            .map(
                |FunctionParameter {
                     name,
                     r#type,
                     type_span,
                 }| TypedFunctionParameter {
                    name,
                    r#type: namespace.resolve_type(r#type, self_type),
                    type_span,
                },
            )
            .collect::<Vec<_>>();
        let mut generic_params_buf_for_error_message = Vec::new();
        for param in parameters.iter() {
            if let Ok(MaybeResolvedType::Partial(PartiallyResolvedType::Generic { ref name })) =
                namespace.resolve_type(*param.r#type, self_type)
            {
                generic_params_buf_for_error_message.push(name.primary_name);
            }
        }
        let comma_separated_generic_params = generic_params_buf_for_error_message.join(", ");
        for TypedFunctionParameter {
            ref r#type,
            type_span,
            ..
        } in parameters.iter()
        {
            if let MaybeResolvedType::Partial(PartiallyResolvedType::Generic { name, .. }) = r#type
            {
                let args_span = parameters.iter().fold(
                    parameters[0].name.span.clone(),
                    |acc, TypedFunctionParameter { type_span, .. }| {
                        crate::utils::join_spans(acc, type_span.clone())
                    },
                );
                if type_parameters
                    .iter()
                    .find(
                        |TypeParameter {
                             name: this_name, ..
                         }| {
                            if let TypeInfo::Custom { name: this_name } = this_name {
                                this_name.primary_name == name.primary_name
                            } else {
                                false
                            }
                        },
                    )
                    .is_none()
                {
                    errors.push(CompileError::TypeParameterNotInTypeScope {
                        name: name.primary_name,
                        span: type_span.clone(),
                        comma_separated_generic_params: comma_separated_generic_params.clone(),
                        fn_name: fn_decl.name.primary_name,
                        args: args_span.as_str().to_string(),
                    });
                }
            }
        }
        // handle the return statement(s)
        let return_statements: Vec<(&TypedExpression, &Span<'sc>)> = body
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
            let convertability = stmt.return_type.is_convertible(
                &return_type,
                span.clone(),
                "Function body's return type does not match up with its return type annotation.",
            );
            match convertability {
                Ok(warning) => {
                    if let Some(warning) = warning {
                        warnings.push(CompileWarning {
                            warning_content: warning,
                            span: span.clone(),
                        });
                    }
                }
                Err(err) => {
                    errors.push(err.into());
                }
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
                if parameters[0].r#type
                    != MaybeResolvedType::Resolved(ResolvedType::UnsignedInteger(
                        IntegerBits::SixtyFour,
                    ))
                {
                    errors.push(CompileError::AbiFunctionRequiresSpecificSignature {
                        span: parameters[0].type_span.clone(),
                    });
                }
                if parameters[1].r#type
                    != MaybeResolvedType::Resolved(ResolvedType::UnsignedInteger(
                        IntegerBits::SixtyFour,
                    ))
                {
                    errors.push(CompileError::AbiFunctionRequiresSpecificSignature {
                        span: parameters[1].type_span.clone(),
                    });
                }
                if parameters[2].r#type != MaybeResolvedType::Resolved(ResolvedType::B256) {
                    errors.push(CompileError::AbiFunctionRequiresSpecificSignature {
                        span: parameters[2].type_span.clone(),
                    });
                }
            } else {
                errors.push(CompileError::AbiFunctionRequiresSpecificSignature {
                    span: parameters[0].type_span.clone(),
                });
            }
        }

        ok(
            TypedFunctionDeclaration {
                name,
                body,
                parameters,
                span: span.clone(),
                return_type,
                type_parameters,
                return_type_span,
                visibility,
                // if this is for a contract, then it is a contract call
                is_contract_call: mode == Mode::ImplAbiFn,
            },
            warnings,
            errors,
        )
    }
}

impl<'sc> TypedTraitFn<'sc> {
    /// This function is used in trait declarations to insert "placeholder" functions
    /// in the methods. This allows the methods to use functions declared in the
    /// interface surface.
    pub(crate) fn to_dummy_func(&self, mode: Mode) -> TypedFunctionDeclaration<'sc> {
        TypedFunctionDeclaration {
            name: self.name.clone(),
            body: TypedCodeBlock {
                contents: vec![],
                whole_block_span: self.name.span.clone(),
            },
            parameters: self.parameters.clone(),
            span: self.name.span.clone(),
            return_type: todo!(
                "type id for self.return_type.clone(), perhaps take the namespace into this fn"
            ),
            return_type_span: self.return_type_span.clone(),
            visibility: Visibility::Public,
            type_parameters: vec![],
            is_contract_call: mode == Mode::ImplAbiFn,
        }
    }
}
