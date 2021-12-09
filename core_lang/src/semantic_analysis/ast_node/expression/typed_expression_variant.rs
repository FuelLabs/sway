use lazy_static::__Deref;

use super::match_branch::TypedMatchBranch;
use super::*;
use crate::parse_tree::AsmOp;
use crate::semantic_analysis::ast_node::*;
use crate::type_engine::*;
use crate::utils::join_spans;
use crate::Ident;

#[derive(Clone, Debug)]
pub(crate) struct ContractCallMetadata<'sc> {
    pub(crate) func_selector: [u8; 4],
    pub(crate) contract_address: Box<TypedExpression<'sc>>,
}

#[derive(Clone, Debug)]
pub(crate) enum TypedExpressionVariant<'sc> {
    Literal(Literal<'sc>),
    FunctionApplication {
        name: CallPath<'sc>,
        arguments: Vec<(Ident<'sc>, TypedExpression<'sc>)>,
        function_body: TypedCodeBlock<'sc>,
        /// If this is `Some(val)` then `val` is the metadata. If this is `None`, then
        /// there is no selector.
        selector: Option<ContractCallMetadata<'sc>>,
    },
    LazyOperator {
        op: LazyOp,
        lhs: Box<TypedExpression<'sc>>,
        rhs: Box<TypedExpression<'sc>>,
    },
    VariableExpression {
        name: Ident<'sc>,
    },
    Unit,
    Array {
        contents: Vec<TypedExpression<'sc>>,
    },
    ArrayIndex {
        prefix: Box<TypedExpression<'sc>>,
        index: Box<TypedExpression<'sc>>,
    },
    StructExpression {
        struct_name: Ident<'sc>,
        fields: Vec<TypedStructExpressionField<'sc>>,
    },
    CodeBlock(TypedCodeBlock<'sc>),
    // a flag that this value will later be provided as a parameter, but is currently unknown
    FunctionParameter,
    ScrutineeParameter,
    IfExp {
        condition: Box<TypedExpression<'sc>>,
        then: Box<TypedExpression<'sc>>,
        r#else: Option<Box<TypedExpression<'sc>>>,
    },
    AsmExpression {
        registers: Vec<TypedAsmRegisterDeclaration<'sc>>,
        body: Vec<AsmOp<'sc>>,
        returns: Option<(AsmRegister, Span<'sc>)>,
        whole_block_span: Span<'sc>,
    },
    // like a variable expression but it has multiple parts,
    // like looking up a field in a struct
    StructFieldAccess {
        prefix: Box<TypedExpression<'sc>>,
        field_to_access: OwnedTypedStructField,
        field_to_access_span: Span<'sc>,
        resolved_type_of_parent: TypeId,
    },
    EnumInstantiation {
        /// for printing
        enum_decl: TypedEnumDeclaration<'sc>,
        /// for printing
        variant_name: Ident<'sc>,
        tag: usize,
        contents: Option<Box<TypedExpression<'sc>>>,
    },
    AbiCast {
        abi_name: CallPath<'sc>,
        address: Box<TypedExpression<'sc>>,
        #[allow(dead_code)]
        // this span may be used for errors in the future, although it is not right now.
        span: Span<'sc>,
    },
    MatchExpression {
        condition: Box<TypedExpression<'sc>>,
        branches: Vec<TypedMatchBranch<'sc>>,
        span: Span<'sc>,
    },
}

#[derive(Clone, Debug)]
pub(crate) struct TypedAsmRegisterDeclaration<'sc> {
    pub(crate) initializer: Option<TypedExpression<'sc>>,
    pub(crate) name: &'sc str,
    pub(crate) name_span: Span<'sc>,
}

impl TypedAsmRegisterDeclaration<'_> {
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        if let Some(ref mut initializer) = self.initializer {
            initializer.copy_types(type_mapping)
        }
    }
}

struct TypedMatchedBranch<'sc> {
    result: TypedExpression<'sc>,
    match_req_map: Vec<(TypedExpression<'sc>, TypedExpression<'sc>)>,
    match_impl_map: Vec<(Ident<'sc>, TypedExpression<'sc>)>,
    branch_span: Span<'sc>,
}

impl<'sc> TypedExpressionVariant<'sc> {
    pub(crate) fn pretty_print(&self) -> String {
        match self {
            TypedExpressionVariant::Literal(lit) => format!(
                "literal {}",
                match lit {
                    Literal::U8(content) => content.to_string(),
                    Literal::U16(content) => content.to_string(),
                    Literal::U32(content) => content.to_string(),
                    Literal::U64(content) => content.to_string(),
                    Literal::String(content) => content.to_string(),
                    Literal::Boolean(content) => content.to_string(),
                    Literal::Byte(content) => content.to_string(),
                    Literal::B256(content) => content
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                }
            ),
            TypedExpressionVariant::FunctionApplication { name, .. } => {
                format!("\"{}\" fn entry", name.suffix.primary_name)
            }
            TypedExpressionVariant::LazyOperator { op, .. } => match op {
                LazyOp::And => "&&".into(),
                LazyOp::Or => "||".into(),
            },
            TypedExpressionVariant::Unit => "unit".into(),
            TypedExpressionVariant::Array { .. } => "array".into(),
            TypedExpressionVariant::ArrayIndex { .. } => "[..]".into(),
            TypedExpressionVariant::StructExpression { struct_name, .. } => {
                format!("\"{}\" struct init", struct_name.primary_name)
            }
            TypedExpressionVariant::CodeBlock(_) => "code block entry".into(),
            TypedExpressionVariant::FunctionParameter => "fn param access".into(),
            TypedExpressionVariant::ScrutineeParameter => "scrutinee param access".into(),
            TypedExpressionVariant::IfExp { .. } => "if exp".into(),
            TypedExpressionVariant::AsmExpression { .. } => "inline asm".into(),
            TypedExpressionVariant::AbiCast { abi_name, .. } => {
                format!("abi cast {}", abi_name.suffix.primary_name)
            }
            TypedExpressionVariant::StructFieldAccess {
                resolved_type_of_parent,
                field_to_access,
                ..
            } => {
                format!(
                    "\"{}.{}\" struct field access",
                    look_up_type_id(*resolved_type_of_parent).friendly_type_str(),
                    field_to_access.name
                )
            }
            TypedExpressionVariant::VariableExpression { name, .. } => {
                format!("\"{}\" variable exp", name.primary_name)
            }
            TypedExpressionVariant::EnumInstantiation {
                tag,
                enum_decl,
                variant_name,
                ..
            } => {
                format!(
                    "{}::{} enum instantiation (tag: {})",
                    enum_decl.name.primary_name, variant_name.primary_name, tag
                )
            }
            TypedExpressionVariant::MatchExpression { .. } => "match exp".into(),
        }
    }
    /// Makes a fresh copy of all type ids in this expression. Used when monomorphizing.
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        use TypedExpressionVariant::*;
        match self {
            Literal(..) => (),
            FunctionApplication {
                arguments,
                function_body,
                ..
            } => {
                arguments
                    .iter_mut()
                    .for_each(|(_ident, expr)| expr.copy_types(type_mapping));
                function_body.copy_types(type_mapping);
            }
            LazyOperator { lhs, rhs, .. } => {
                (*lhs).copy_types(type_mapping);
                (*rhs).copy_types(type_mapping);
            }
            VariableExpression { .. } => (),
            Unit => (),
            Array { contents } => contents.iter_mut().for_each(|x| x.copy_types(type_mapping)),
            ArrayIndex { prefix, index } => {
                (*prefix).copy_types(type_mapping);
                (*index).copy_types(type_mapping);
            }
            StructExpression { fields, .. } => {
                fields.iter_mut().for_each(|x| x.copy_types(type_mapping))
            }
            CodeBlock(block) => {
                block.copy_types(type_mapping);
            }
            FunctionParameter => (),
            ScrutineeParameter => (),
            IfExp {
                condition,
                then,
                r#else,
            } => {
                condition.copy_types(type_mapping);
                then.copy_types(type_mapping);
                if let Some(ref mut r#else) = r#else {
                    r#else.copy_types(type_mapping);
                }
            }
            MatchExpression {
                condition,
                branches,
                ..
            } => {
                condition.copy_types(type_mapping);
                branches.iter_mut().for_each(|x| x.copy_types(type_mapping));
            }
            AsmExpression {
                registers, //: Vec<TypedAsmRegisterDeclaration<'sc>>,
                ..
            } => {
                registers
                    .iter_mut()
                    .for_each(|x| x.copy_types(type_mapping));
            }
            // like a variable expression but it has multiple parts,
            // like looking up a field in a struct
            StructFieldAccess {
                prefix,
                field_to_access,
                ref mut resolved_type_of_parent,
                ..
            } => {
                *resolved_type_of_parent = if let Some(matching_id) =
                    look_up_type_id(*resolved_type_of_parent).matches_type_parameter(&type_mapping)
                {
                    insert_type(TypeInfo::Ref(matching_id))
                } else {
                    insert_type(look_up_type_id_raw(*resolved_type_of_parent))
                };

                field_to_access.copy_types(type_mapping);
                prefix.copy_types(type_mapping);
            }
            EnumInstantiation {
                enum_decl,
                contents,
                ..
            } => {
                enum_decl.copy_types(type_mapping);
                if let Some(ref mut contents) = contents {
                    contents.copy_types(type_mapping)
                };
            }
            AbiCast { address, .. } => address.copy_types(type_mapping),
        }
    }

    pub(crate) fn desugar(&self) -> CompileResult<'sc, Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let variant = match self {
            TypedExpressionVariant::VariableExpression { name } => {
                TypedExpressionVariant::VariableExpression { name: name.clone() }
            }
            TypedExpressionVariant::Literal(lit) => TypedExpressionVariant::Literal(lit.to_owned()),
            TypedExpressionVariant::MatchExpression {
                condition,
                branches,
                span,
            } => {
                check!(
                    Self::desugar_match_expression(&*condition, branches, span),
                    return err(warnings, errors),
                    warnings,
                    errors
                )
            }
            TypedExpressionVariant::CodeBlock(code_block) => {
                let code_block = check!(
                    code_block.desugar(),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                TypedExpressionVariant::CodeBlock(code_block)
            }
            variant => unimplemented!("{:?}", variant),
        };
        ok(variant, warnings, errors)
    }

    fn desugar_match_expression(
        primary_expression: &TypedExpression<'sc>,
        branches: &Vec<TypedMatchBranch<'sc>>,
        match_span: &Span<'sc>,
    ) -> CompileResult<'sc, Self> {
        let mut warnings = vec![];
        let mut errors = vec![];

        // 1. Assemble the "matched branches."
        let mut matched_branches = vec![];
        for TypedMatchBranch {
            condition,
            result,
            span: branch_span,
        } in branches.iter()
        {
            let matches = match condition {
                TypedMatchCondition::CatchAll(_) => Some((vec![], vec![])),
                TypedMatchCondition::Scrutinee(scrutinee) => {
                    matcher::matcher(primary_expression, scrutinee)
                }
            };
            match matches {
                Some((match_req_map, match_impl_map)) => {
                    matched_branches.push(TypedMatchedBranch {
                        result: check!(
                            result.desugar(),
                            return err(warnings, errors),
                            warnings,
                            errors
                        ),
                        match_req_map,
                        match_impl_map,
                        branch_span: branch_span.to_owned(),
                    });
                }
                None => unimplemented!("implement proper error handling"),
            }
        }

        // 2. Assemble the possibly nested giant if statement using the matched branches.
        let mut if_statement: Option<TypedExpressionVariant> = None;
        for TypedMatchedBranch {
            result,
            match_req_map,
            match_impl_map,
            branch_span,
        } in matched_branches.iter().rev()
        {
            // 2a. Assemble the conditional that goes in the if primary expression.
            let mut conditional = None;
            for (left_req, right_req) in match_req_map.iter() {
                let joined_span = join_spans(left_req.span.clone(), right_req.span.clone());
                /*
                let condition = Expression::MethodApplication {
                    method_name: MethodName::FromType {
                        call_path: CallPath {
                            prefixes: vec![
                                Ident {
                                    primary_name: "std",
                                    span: joined_span.clone(),
                                },
                                Ident {
                                    primary_name: "ops",
                                    span: joined_span.clone(),
                                },
                            ],
                            suffix: Op {
                                op_variant: OpVariant::Equals,
                                span: joined_span.clone(),
                            }
                            .to_var_name(),
                        },
                        type_name: None,
                        is_absolute: true,
                    },
                    arguments: vec![left_req.to_owned(), right_req.to_owned()],
                    span: joined_span,
                };
                */
                let condition = TypedExpression {
                    expression: todo!(),
                    span: joined_span,
                    return_type: crate::type_engine::insert_type(TypeInfo::Boolean),
                    is_constant: IsConstant::No,
                };
                match conditional {
                    None => {
                        conditional = Some(condition);
                    }
                    Some(the_conditional) => {
                        let joined_span = join_spans(the_conditional.span, condition.span);
                        conditional = Some(TypedExpression {
                            expression: TypedExpressionVariant::LazyOperator {
                                op: crate::LazyOp::And,
                                lhs: Box::new(the_conditional.clone()),
                                rhs: Box::new(condition.clone()),
                            },
                            span: joined_span,
                            return_type: crate::type_engine::insert_type(TypeInfo::Boolean),
                            is_constant: IsConstant::No,
                        })
                    }
                }
            }

            // 2b. Assemble the statements that go inside of the body of the if expression
            let mut code_block_stmts = vec![];
            let mut code_block_stmts_span = None;
            for (left_impl, right_impl) in match_impl_map.iter() {
                let decl = TypedDeclaration::VariableDeclaration(TypedVariableDeclaration {
                    name: left_impl.clone(),
                    is_mutable: false,
                    body: right_impl.clone(),
                    type_ascription: right_impl.return_type,
                });
                let new_span = join_spans(left_impl.span.clone(), right_impl.span.clone());
                code_block_stmts.push(TypedAstNode {
                    content: TypedAstNodeContent::Declaration(decl),
                    span: new_span.clone(),
                });
                code_block_stmts_span = match code_block_stmts_span {
                    None => Some(new_span),
                    Some(old_span) => Some(join_spans(old_span, new_span)),
                };
            }
            match result {
                TypedExpression {
                    expression:
                        TypedExpressionVariant::CodeBlock(TypedCodeBlock {
                            contents,
                            whole_block_span,
                        }),
                    ..
                } => {
                    let mut contents = contents.clone();
                    code_block_stmts.append(&mut contents);
                    code_block_stmts_span = match code_block_stmts_span {
                        None => Some(whole_block_span.to_owned()),
                        Some(old_span) => Some(join_spans(old_span, whole_block_span.clone())),
                    }
                }
                result => {
                    code_block_stmts.push(TypedAstNode {
                        content: TypedAstNodeContent::Expression(result.clone()),
                        span: result.span.clone(),
                    });
                    code_block_stmts_span = match code_block_stmts_span {
                        None => Some(result.span.clone()),
                        Some(old_span) => Some(join_spans(old_span, result.span.clone())),
                    }
                }
            }
            let code_block_stmts_span = match code_block_stmts_span {
                None => branch_span.clone(),
                Some(code_block_stmts_span) => code_block_stmts_span,
            };
            let code_block = TypedExpression {
                expression: TypedExpressionVariant::CodeBlock(TypedCodeBlock {
                    contents: code_block_stmts,
                    whole_block_span: code_block_stmts_span.clone(),
                }),
                return_type: result.return_type,
                is_constant: IsConstant::No,
                span: code_block_stmts_span,
            };

            // 2c. Assemble the giant if statement.
            match if_statement {
                // if this is the first branch to be packed into the if...
                None => {
                    if_statement = match conditional {
                        None => Some(code_block.expression),
                        Some(conditional) => Some(TypedExpressionVariant::IfExp {
                            condition: Box::new(conditional),
                            then: Box::new(code_block),
                            r#else: None,
                        }),
                    };
                }
                // if this is the second branch to be packed into the if...
                Some(TypedExpressionVariant::CodeBlock(right_block)) => {
                    let right = TypedExpression {
                        expression: TypedExpressionVariant::CodeBlock(right_block.clone()),
                        is_constant: IsConstant::No,
                        span: right_block.whole_block_span,
                        return_type: result.return_type,
                    };
                    let variant = match conditional {
                        None => TypedExpressionVariant::IfExp {
                            condition: Box::new(TypedExpression {
                                expression: TypedExpressionVariant::Literal(Literal::Boolean(true)),
                                is_constant: IsConstant::No,
                                span: branch_span.clone(),
                                return_type: crate::type_engine::insert_type(TypeInfo::Boolean),
                            }),
                            then: Box::new(code_block.clone()),
                            r#else: Some(Box::new(right.clone())),
                        },
                        Some(the_conditional) => TypedExpressionVariant::IfExp {
                            condition: Box::new(the_conditional),
                            then: Box::new(code_block.clone()),
                            r#else: Some(Box::new(right.clone())),
                        },
                    };
                    if_statement = Some(variant);
                }
                // if this is the >2 branch to be packed into the if...
                Some(TypedExpressionVariant::IfExp {
                    condition,
                    then,
                    r#else,
                }) => {
                    let joined_span = match r#else.clone() {
                        None => join_spans(condition.span.clone(), then.span.clone()),
                        Some(r#else) => join_spans(
                            condition.span.clone(),
                            join_spans(then.span.clone(), r#else.span.clone()),
                        ),
                    };
                    let right = TypedExpression {
                        expression: TypedExpressionVariant::IfExp {
                            condition,
                            then,
                            r#else,
                        },
                        is_constant: IsConstant::No,
                        span: joined_span,
                        return_type: result.return_type,
                    };
                    let variant = TypedExpressionVariant::IfExp {
                        condition: Box::new(conditional.unwrap()),
                        then: Box::new(code_block),
                        r#else: Some(Box::new(right)),
                    };
                    if_statement = Some(variant);
                }
                _ => unimplemented!(),
            }
        }

        // 3. Return!
        match if_statement {
            None => err(warnings, errors),
            Some(if_statement) => ok(if_statement, warnings, errors),
        }
    }
}
