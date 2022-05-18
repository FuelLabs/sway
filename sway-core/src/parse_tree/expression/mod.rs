use crate::{
    build_config::BuildConfig,
    error::*,
    parse_tree::{ident, CallPath, Literal},
    type_engine::TypeInfo,
    AstNode, AstNodeContent, CodeBlock, Declaration, TypeArgument, VariableDeclaration,
};
use sway_types::{ident::Ident, Span};

mod asm;
mod match_branch;
mod matcher;
mod method_name;
mod scrutinee;
pub(crate) use asm::*;
pub(crate) use match_branch::MatchBranch;
use matcher::matcher;
pub use method_name::MethodName;
pub(crate) use scrutinee::{Scrutinee, StructScrutineeField};

/// Represents a parsed, but not yet type checked, [Expression](https://en.wikipedia.org/wiki/Expression_(computer_science)).
#[derive(Debug, Clone)]
pub enum Expression {
    Literal {
        value: Literal,
        span: Span,
    },
    FunctionApplication {
        name: CallPath,
        arguments: Vec<Expression>,
        type_arguments: Vec<TypeArgument>,
        span: Span,
    },
    LazyOperator {
        op: LazyOp,
        lhs: Box<Expression>,
        rhs: Box<Expression>,
        span: Span,
    },
    VariableExpression {
        name: Ident,
        span: Span,
    },
    Tuple {
        fields: Vec<Expression>,
        span: Span,
    },
    TupleIndex {
        prefix: Box<Expression>,
        index: usize,
        index_span: Span,
        span: Span,
    },
    Array {
        contents: Vec<Expression>,
        span: Span,
    },
    StructExpression {
        struct_name: CallPath,
        type_arguments: Vec<TypeArgument>,
        fields: Vec<StructExpressionField>,
        span: Span,
    },
    CodeBlock {
        contents: CodeBlock,
        span: Span,
    },
    IfExp {
        condition: Box<Expression>,
        then: Box<Expression>,
        r#else: Option<Box<Expression>>,
        span: Span,
    },
    MatchExp {
        if_exp: Box<Expression>,
        cases_covered: Vec<Scrutinee>,
        span: Span,
    },
    // separated into other struct for parsing reasons
    AsmExpression {
        span: Span,
        asm: AsmExpression,
    },
    MethodApplication {
        method_name: MethodName,
        contract_call_params: Vec<StructExpressionField>,
        arguments: Vec<Expression>,
        type_arguments: Vec<TypeArgument>,
        span: Span,
    },
    /// A _subfield expression_ is anything of the form:
    /// ```ignore
    /// <ident>.<ident>
    /// ```
    ///
    SubfieldExpression {
        prefix: Box<Expression>,
        span: Span,
        field_to_access: Ident,
    },
    /// A _delineated path_ is anything of the form:
    /// ```ignore
    /// <ident>::<ident>
    /// ```
    /// Where there are `n >= 2` idents.
    /// These could be either enum variant constructions, or they could be
    /// references to some sort of module in the module tree.
    /// For example, a reference to a module:
    /// ```ignore
    /// std::ops::add
    /// ```
    ///
    /// And, an enum declaration:
    /// ```ignore
    /// enum MyEnum {
    ///   Variant1,
    ///   Variant2
    /// }
    ///
    /// MyEnum::Variant1
    /// ```
    DelineatedPath {
        call_path: CallPath,
        args: Vec<Expression>,
        span: Span,
        type_arguments: Vec<TypeArgument>,
    },
    /// A cast of a hash to an ABI for calling a contract.
    AbiCast {
        abi_name: CallPath,
        address: Box<Expression>,
        span: Span,
    },
    ArrayIndex {
        prefix: Box<Expression>,
        index: Box<Expression>,
        span: Span,
    },
    /// This variant serves as a stand-in for parsing-level match expression desugaring.
    /// Because types cannot be known at parsing-time, a desugared struct or enum gets
    /// special cased into this variant. During type checking, this variant is removed
    /// as is replaced with the corresponding field or argument access (given that the
    /// expression inside of the delayed resolution has the appropriate struct or enum
    /// type)
    DelayedMatchTypeResolution {
        variant: DelayedResolutionVariant,
        span: Span,
    },
    StorageAccess {
        field_names: Vec<Ident>,
        span: Span,
    },
    IfLet {
        scrutinee: Scrutinee,
        expr: Box<Expression>,
        then: CodeBlock,
        r#else: Option<Box<Expression>>,
        span: Span,
    },
    SizeOfVal {
        exp: Box<Expression>,
        span: Span,
    },
    BuiltinGetTypeProperty {
        builtin: BuiltinProperty,
        type_name: TypeInfo,
        type_span: Span,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BuiltinProperty {
    SizeOfType,
    IsRefType,
}

#[derive(Debug, Clone)]
pub enum DelayedResolutionVariant {
    StructField(DelayedStructFieldResolution),
    EnumVariant(DelayedEnumVariantResolution),
    TupleVariant(DelayedTupleVariantResolution),
}

/// During type checking, this gets replaced with struct field access.
#[derive(Debug, Clone)]
pub struct DelayedStructFieldResolution {
    pub exp: Box<Expression>,
    pub struct_name: Ident,
    pub field: Ident,
}

/// During type checking, this gets replaced with an if let, maybe, although that's not yet been
/// implemented.
#[derive(Debug, Clone)]
pub struct DelayedEnumVariantResolution {
    pub exp: Box<Expression>,
    pub call_path: CallPath,
    pub arg_num: usize,
}

/// During type checking, this gets replaced with tuple arg access.
#[derive(Debug, Clone)]
pub struct DelayedTupleVariantResolution {
    pub exp: Box<Expression>,
    pub elem_num: usize,
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub enum LazyOp {
    And,
    Or,
}

#[derive(Debug, Clone)]
pub struct StructExpressionField {
    pub name: Ident,
    pub value: Expression,
    pub(crate) span: Span,
}

impl Expression {
    pub(crate) fn core_ops_eq(arguments: Vec<Expression>, span: Span) -> Expression {
        Expression::MethodApplication {
            method_name: MethodName::FromType {
                call_path: CallPath {
                    prefixes: vec![
                        Ident::new_with_override("core", span.clone()),
                        Ident::new_with_override("ops", span.clone()),
                    ],
                    suffix: Op {
                        op_variant: OpVariant::Equals,
                        span: span.clone(),
                    }
                    .to_var_name(),
                    is_absolute: true,
                },
                type_name: None,
                type_name_span: None,
            },
            contract_call_params: vec![],
            arguments,
            type_arguments: vec![],
            span,
        }
    }

    pub(crate) fn span(&self) -> Span {
        use Expression::*;
        (match self {
            Literal { span, .. } => span,
            FunctionApplication { span, .. } => span,
            LazyOperator { span, .. } => span,
            VariableExpression { span, .. } => span,
            Tuple { span, .. } => span,
            TupleIndex { span, .. } => span,
            Array { span, .. } => span,
            StructExpression { span, .. } => span,
            CodeBlock { span, .. } => span,
            IfExp { span, .. } => span,
            MatchExp { span, .. } => span,
            AsmExpression { span, .. } => span,
            MethodApplication { span, .. } => span,
            SubfieldExpression { span, .. } => span,
            DelineatedPath { span, .. } => span,
            AbiCast { span, .. } => span,
            ArrayIndex { span, .. } => span,
            DelayedMatchTypeResolution { span, .. } => span,
            StorageAccess { span, .. } => span,
            IfLet { span, .. } => span,
            SizeOfVal { span, .. } => span,
            BuiltinGetTypeProperty { span, .. } => span,
        })
        .clone()
    }
}

#[derive(Debug)]
pub(crate) struct Op {
    pub span: Span,
    pub op_variant: OpVariant,
}

impl Op {
    pub fn to_var_name(&self) -> Ident {
        Ident::new_with_override(self.op_variant.as_str(), self.span.clone())
    }
}

#[derive(Debug)]
pub enum OpVariant {
    Add,
    Subtract,
    Divide,
    Multiply,
    Modulo,
    Or,
    And,
    Equals,
    NotEquals,
    Xor,
    BinaryOr,
    BinaryAnd,
    GreaterThan,
    LessThan,
    GreaterThanOrEqualTo,
    LessThanOrEqualTo,
}

impl OpVariant {
    fn as_str(&self) -> &'static str {
        use OpVariant::*;
        match self {
            Add => "add",
            Subtract => "subtract",
            Divide => "divide",
            Multiply => "multiply",
            Modulo => "modulo",
            Or => "$or$",
            And => "$and$",
            Equals => "eq",
            NotEquals => "neq",
            Xor => "xor",
            BinaryOr => "binary_or",
            BinaryAnd => "binary_and",
            GreaterThan => "gt",
            LessThan => "lt",
            LessThanOrEqualTo => "le",
            GreaterThanOrEqualTo => "ge",
        }
    }
}

struct MatchedBranch {
    result: Expression,
    match_req_map: Vec<(Expression, Expression)>,
    match_impl_map: Vec<(Ident, Expression)>,
    branch_span: Span,
}

/// This algorithm desugars match expressions to if statements.
///
/// Given the following example:
///
/// ```ignore
/// struct Point {
///     x: u64,
///     y: u64
/// }
///
/// let p = Point {
///     x: 42,
///     y: 24
/// };
///
/// match p {
///     Point { x, y: 5 } => { x },
///     Point { x, y: 24 } => { x },
///     _ => 0
/// }
/// ```
///
/// The resulting if statement would look roughly like this:
///
/// ```ignore
/// let NEW_NAME = p;
/// if NEW_NAME.y==5 {
///     let x = 42;
///     x
/// } else if NEW_NAME.y==42 {
///     let x = 42;
///     x
/// } else {
///     0
/// }
/// ```
///
/// The steps of the algorithm can roughly be broken down into:
///
/// 0. Create a VariableDeclaration that assigns the primary expression to a variable.
/// 1. Assemble the "matched branches."
/// 2. Assemble the possibly nested giant if statement using the matched branches.
///     2a. Assemble the conditional that goes in the if primary expression.
///     2b. Assemble the statements that go inside of the body of the if expression
///     2c. Assemble the giant if statement.
/// 3. Return!
pub(crate) fn desugar_match_expression(
    primary_expression: &Expression,
    branches: Vec<MatchBranch>,
    config: Option<&BuildConfig>,
) -> CompileResult<(Expression, Ident, Vec<Scrutinee>)> {
    let mut errors = vec![];
    let mut warnings = vec![];

    // 0. Create a VariableDeclaration that assigns the primary expression to a variable.
    let var_decl_span = primary_expression.span();
    let var_decl_name = ident::random_name(var_decl_span.clone(), config);
    let var_decl_exp = Expression::VariableExpression {
        name: var_decl_name.clone(),
        span: var_decl_span,
    };

    // 1. Assemble the "matched branches."
    let mut matched_branches = vec![];
    for MatchBranch {
        condition,
        result,
        span: branch_span,
    } in branches.iter()
    {
        let (match_req_map, match_impl_map) = check!(
            matcher(&var_decl_exp, condition),
            return err(warnings, errors),
            warnings,
            errors
        );
        matched_branches.push(MatchedBranch {
            result: result.to_owned(),
            match_req_map,
            match_impl_map,
            branch_span: branch_span.to_owned(),
        });
    }

    // 2. Assemble the possibly nested giant if statement using the matched branches.
    let mut if_statement = None;
    for MatchedBranch {
        result,
        match_req_map,
        match_impl_map,
        branch_span,
    } in matched_branches.iter().rev()
    {
        // 2a. Assemble the conditional that goes in the if primary expression.
        let mut conditional = None;
        for (left_req, right_req) in match_req_map.iter() {
            let joined_span = Span::join(left_req.clone().span(), right_req.clone().span());
            let condition = Expression::core_ops_eq(
                vec![left_req.to_owned(), right_req.to_owned()],
                joined_span,
            );
            match conditional {
                None => {
                    conditional = Some(condition);
                }
                Some(the_conditional) => {
                    conditional = Some(Expression::LazyOperator {
                        op: crate::LazyOp::And,
                        lhs: Box::new(the_conditional.clone()),
                        rhs: Box::new(condition.clone()),
                        span: Span::join(the_conditional.span(), condition.span()),
                    });
                }
            }
        }

        // 2b. Assemble the statements that go inside of the body of the if expression
        let mut code_block_stmts = vec![];
        let mut code_block_stmts_span = None;
        for (left_impl, right_impl) in match_impl_map.iter() {
            let decl = Declaration::VariableDeclaration(VariableDeclaration {
                name: left_impl.clone(),
                is_mutable: false,
                body: right_impl.clone(),
                type_ascription: TypeInfo::Unknown,
                type_ascription_span: None,
            });
            let new_span = Span::join(left_impl.span().clone(), right_impl.span());
            code_block_stmts.push(AstNode {
                content: AstNodeContent::Declaration(decl),
                span: new_span.clone(),
            });
            code_block_stmts_span = match code_block_stmts_span {
                None => Some(new_span),
                Some(old_span) => Some(Span::join(old_span, new_span)),
            };
        }
        match result {
            Expression::CodeBlock {
                contents:
                    CodeBlock {
                        contents,
                        whole_block_span,
                    },
                span: _,
            } => {
                let mut contents = contents.clone();
                code_block_stmts.append(&mut contents);
                code_block_stmts_span = match code_block_stmts_span {
                    None => Some(whole_block_span.clone()),
                    Some(old_span) => Some(Span::join(old_span, whole_block_span.clone())),
                };
            }
            result => {
                code_block_stmts.push(AstNode {
                    content: AstNodeContent::Expression(result.clone()),
                    span: result.span(),
                });
                code_block_stmts_span = match code_block_stmts_span {
                    None => Some(result.span()),
                    Some(old_span) => Some(Span::join(old_span, result.span())),
                };
            }
        }
        let code_block_stmts_span = match code_block_stmts_span {
            None => branch_span.clone(),
            Some(span) => span,
        };
        let code_block = Expression::CodeBlock {
            contents: CodeBlock {
                contents: code_block_stmts.clone(),
                whole_block_span: code_block_stmts_span.clone(),
            },
            span: code_block_stmts_span,
        };

        // 2c. Assemble the giant if statement.
        match if_statement {
            None => {
                if_statement = match conditional {
                    None => Some(code_block),
                    Some(conditional) => Some(Expression::IfExp {
                        condition: Box::new(conditional.clone()),
                        then: Box::new(code_block.clone()),
                        r#else: None,
                        span: Span::join(conditional.span(), code_block.span()),
                    }),
                };
            }
            Some(Expression::CodeBlock {
                contents: right_block,
                span: exp_span,
            }) => {
                let right = Expression::CodeBlock {
                    contents: right_block,
                    span: exp_span,
                };
                if_statement = match conditional {
                    None => Some(Expression::IfExp {
                        condition: Box::new(Expression::Literal {
                            value: Literal::Boolean(true),
                            span: branch_span.clone(),
                        }),
                        then: Box::new(code_block.clone()),
                        r#else: Some(Box::new(right.clone())),
                        span: Span::join(code_block.clone().span(), right.clone().span()),
                    }),
                    Some(the_conditional) => Some(Expression::IfExp {
                        condition: Box::new(the_conditional),
                        then: Box::new(code_block.clone()),
                        r#else: Some(Box::new(right.clone())),
                        span: Span::join(code_block.clone().span(), right.clone().span()),
                    }),
                };
            }
            Some(Expression::IfExp {
                condition,
                then,
                r#else,
                span: exp_span,
            }) => {
                if_statement = Some(Expression::IfExp {
                    condition: Box::new(conditional.unwrap()),
                    then: Box::new(code_block.clone()),
                    r#else: Some(Box::new(Expression::IfExp {
                        condition,
                        then,
                        r#else,
                        span: exp_span.clone(),
                    })),
                    span: Span::join(code_block.clone().span(), exp_span),
                });
            }
            Some(if_statement) => {
                eprintln!("Unimplemented if_statement_pattern: {:?}", if_statement,);
                errors.push(CompileError::Unimplemented(
                    "this desugared if expression pattern is not implemented",
                    if_statement.span(),
                ));
                // construct unit expression for error recovery
                let exp = Expression::Tuple {
                    fields: vec![],
                    span: if_statement.span(),
                };
                return ok((exp, var_decl_name, vec![]), warnings, errors);
            }
        }
    }

    // 3. Return!
    let cases_covered = branches
        .into_iter()
        .map(|x| x.condition)
        .collect::<Vec<_>>();
    match if_statement {
        None => err(vec![], vec![]),
        Some(if_statement) => ok(
            (if_statement, var_decl_name, cases_covered),
            warnings,
            errors,
        ),
    }
}
