//! Contains all the code related to parsing Sway source code.
mod code_block;
pub mod declaration;
mod expression;
mod include_statement;
mod module;
mod program;
mod use_statement;

pub use code_block::*;
pub use declaration::*;
pub use expression::*;
pub use include_statement::IncludeStatement;
pub use module::{ParseModule, ParseSubmodule};
pub use program::{ParseProgram, TreeType};
use sway_ast::Intrinsic;
use sway_error::handler::ErrorEmitted;
pub use use_statement::{ImportType, UseStatement};

use sway_types::{span::Span, BaseIdent, Ident};

use crate::{
    decl_engine::parsed_engine::ParsedDeclEngineInsert, Engines, TypeArgs, TypeArgument,
    TypeBinding, TypeInfo,
};

use super::{ty::TyFunctionDecl, CallPath};

/// Represents some exportable information that results from compiling some
/// Sway source code.
#[derive(Debug, Clone)]
pub struct ParseTree {
    /// The untyped AST nodes that constitute this tree's root nodes.
    pub root_nodes: Vec<AstNode>,
    /// The [Span] of the entire tree.
    pub span: Span,
}

/// A single [AstNode] represents a node in the parse tree. Note that [AstNode]
/// is a recursive type and can contain other [AstNode], thus populating the tree.
#[derive(Debug, Clone)]
pub struct AstNode {
    /// The content of this ast node, which could be any control flow structure or other
    /// basic organizational component.
    pub content: AstNodeContent,
    /// The [Span] representing this entire [AstNode].
    pub span: Span,
}

impl AstNode {
    pub fn return_expr(value: Expression) -> AstNode {
        AstNode {
            content: AstNodeContent::Expression(Expression {
                kind: ExpressionKind::Return(Box::new(value)),
                span: Span::dummy(),
            }),
            span: Span::dummy(),
        }
    }

    pub fn implicit_return_expr(value: Expression) -> AstNode {
        AstNode {
            content: AstNodeContent::Expression(Expression {
                kind: ExpressionKind::ImplicitReturn(Box::new(value)),
                span: Span::dummy(),
            }),
            span: Span::dummy(),
        }
    }

    pub fn match_branch(value: Expression, branches: Vec<MatchBranch>) -> AstNode {
        AstNode {
            content: AstNodeContent::Expression(Expression::match_branch(value, branches)),
            span: Span::dummy(),
        }
    }

    pub fn variable_declaration(
        engines: &Engines,
        name: BaseIdent,
        body: Expression,
        is_mutable: bool,
    ) -> Self {
        let unknown = engines.te().insert(engines, TypeInfo::Unknown, None);
        let var_decl = engines.pe().insert(VariableDeclaration {
            name,
            type_ascription: TypeArgument {
                type_id: unknown,
                initial_type_id: unknown,
                span: Span::dummy(),
                call_path_tree: None,
            },
            body,
            is_mutable,
        });
        AstNode {
            content: AstNodeContent::Declaration(Declaration::VariableDeclaration(var_decl)),
            span: Span::dummy(),
        }
    }

    pub fn typed_variable_declaration(
        engines: &Engines,
        name: BaseIdent,
        type_ascription: TypeArgument,
        body: Expression,
        is_mutable: bool,
    ) -> Self {
        let var_decl = engines.pe().insert(VariableDeclaration {
            name,
            type_ascription,
            body,
            is_mutable,
        });
        AstNode {
            content: AstNodeContent::Declaration(Declaration::VariableDeclaration(var_decl)),
            span: Span::dummy(),
        }
    }

    pub fn call_method(method_name: BaseIdent, arguments: Vec<Expression>) -> Self {
        let expr = Expression::call_method(method_name, arguments);
        AstNode {
            content: AstNodeContent::Expression(expr),
            span: Span::dummy(),
        }
    }

    pub fn call_function_with_suffix(suffix: BaseIdent, arguments: Vec<Expression>) -> Self {
        AstNode {
            content: AstNodeContent::Expression(Expression::call_function_with_suffix(
                suffix, arguments,
            )),
            span: Span::dummy(),
        }
    }

    pub fn retd(ptr: Expression, len: Expression) -> Self {
        AstNode {
            content: AstNodeContent::Expression(Expression::retd(ptr, len)),
            span: Span::dummy(),
        }
    }

    pub fn retd_addr_of_variable(var: BaseIdent, len: Expression) -> Self {
        AstNode {
            content: AstNodeContent::Expression(Expression::retd_addr_of_variable(var, len)),
            span: Span::dummy(),
        }
    }

    fn call_fn(expr: Expression, name: &str) -> Expression {
        Expression {
            kind: ExpressionKind::MethodApplication(Box::new(MethodApplicationExpression {
                method_name_binding: TypeBinding {
                    inner: MethodName::FromModule {
                        method_name: Ident::new_no_span(name.to_string()),
                    },
                    type_arguments: TypeArgs::Regular(vec![]),
                    span: Span::dummy(),
                },
                contract_call_params: vec![],
                arguments: vec![expr],
            })),
            span: Span::dummy(),
        }
    }

    fn call_encode(arg: Expression) -> Expression {
        Expression {
            kind: ExpressionKind::FunctionApplication(Box::new(FunctionApplicationExpression {
                call_path_binding: TypeBinding {
                    inner: CallPath {
                        prefixes: vec![],
                        suffix: Ident::new_no_span("encode".into()),
                        is_absolute: false,
                    },
                    type_arguments: TypeArgs::Regular(vec![]),
                    span: Span::dummy(),
                },
                arguments: vec![arg],
            })),
            span: Span::dummy(),
        }
    }

    fn arguments_type(engines: &Engines, decl: &TyFunctionDecl) -> Option<TypeArgument> {
        if decl.parameters.is_empty() {
            return None;
        }

        let types: Vec<_> = decl
            .parameters
            .iter()
            .map(|p| p.type_argument.clone())
            .collect();

        if types.len() == 1 {
            return types.into_iter().next();
        }

        let type_id = engines.te().insert(engines, TypeInfo::Tuple(types), None);
        Some(TypeArgument {
            type_id,
            initial_type_id: type_id,
            span: Span::dummy(),
            call_path_tree: None,
        })
    }

    fn decode_script_data(_engines: &Engines, args_type: TypeArgument) -> Expression {
        Expression {
            kind: ExpressionKind::FunctionApplication(Box::new(FunctionApplicationExpression {
                call_path_binding: TypeBinding {
                    inner: CallPath {
                        prefixes: vec![],
                        suffix: Ident::new_no_span("decode_script_data".into()),
                        is_absolute: false,
                    },
                    type_arguments: TypeArgs::Regular(vec![args_type]),
                    span: Span::dummy(),
                },
                arguments: vec![],
            })),
            span: Span::dummy(),
        }
    }

    fn arguments_as_expressions(var: BaseIdent, decl: &TyFunctionDecl) -> Vec<Expression> {
        if decl.parameters.len() == 1 {
            decl.parameters
                .iter()
                .enumerate()
                .map(|(_, _p)| Expression::ambiguous_variable_expression(var.clone()))
                .collect()
        } else {
            decl.parameters
                .iter()
                .enumerate()
                .map(|(idx, _p)| Expression {
                    kind: ExpressionKind::TupleIndex(TupleIndexExpression {
                        prefix: Box::new(Expression {
                            kind: ExpressionKind::AmbiguousVariableExpression(var.clone()),
                            span: Span::dummy(),
                        }),
                        index: idx,
                        index_span: Span::dummy(),
                    }),
                    span: Span::dummy(),
                })
                .collect()
        }
    }

    pub fn push_decode_script_data_as_fn_args(
        engines: &Engines,
        contents: &mut Vec<AstNode>,
        var: BaseIdent,
        decl: &TyFunctionDecl,
    ) -> Vec<Expression> {
        if let Some(args_type) = Self::arguments_type(engines, decl) {
            contents.push(AstNode::typed_variable_declaration(
                engines,
                var.clone(),
                args_type.clone(),
                Self::decode_script_data(engines, args_type),
                false,
            ));

            Self::arguments_as_expressions(var, decl)
        } else {
            vec![]
        }
    }

    pub fn push_encode_and_return(
        engines: &Engines,
        contents: &mut Vec<AstNode>,
        var: BaseIdent,
        value: Expression,
    ) {
        contents.push(AstNode::variable_declaration(
            engines,
            var.clone(),
            Self::call_encode(value),
            false,
        ));
        contents.push(AstNode {
            content: AstNodeContent::Expression(Expression {
                kind: ExpressionKind::IntrinsicFunction(IntrinsicFunctionExpression {
                    name: Ident::new_no_span("__contract_ret".to_string()),
                    kind_binding: TypeBinding {
                        inner: Intrinsic::ContractRet,
                        type_arguments: TypeArgs::Regular(vec![]),
                        span: Span::dummy(),
                    },
                    arguments: vec![
                        Self::call_fn(
                            Expression {
                                kind: ExpressionKind::AmbiguousVariableExpression(var.clone()),
                                span: Span::dummy(),
                            },
                            "ptr",
                        ),
                        Self::call_fn(
                            Expression {
                                kind: ExpressionKind::AmbiguousVariableExpression(var.clone()),
                                span: Span::dummy(),
                            },
                            "number_of_bytes",
                        ),
                    ],
                }),
                span: Span::dummy(),
            }),
            span: Span::dummy(),
        });
    }
}

/// Represents the various structures that constitute a Sway program.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum AstNodeContent {
    /// A statement of the form `use foo::bar;` or `use ::foo::bar;`
    UseStatement(UseStatement),
    /// Any type of declaration, of which there are quite a few. See [Declaration] for more details
    /// on the possible variants.
    Declaration(Declaration),
    /// Any type of expression, of which there are quite a few. See [Expression] for more details.
    Expression(Expression),
    /// A statement of the form `mod foo::bar;` which imports/includes another source file.
    IncludeStatement(IncludeStatement),
    /// A malformed statement.
    ///
    /// Used for parser recovery when we cannot form a more specific node.
    /// The list of `Span`s are for consumption by the LSP and are,
    /// when joined, the same as that stored in `statement.span`.
    Error(Box<[Span]>, ErrorEmitted),
}

impl ParseTree {
    /// Excludes all test functions from the parse tree.
    pub(crate) fn exclude_tests(&mut self, engines: &Engines) {
        self.root_nodes.retain(|node| !node.is_test(engines));
    }
}

impl AstNode {
    /// Checks if this `AstNode` is a test.
    pub(crate) fn is_test(&self, engines: &Engines) -> bool {
        if let AstNodeContent::Declaration(decl) = &self.content {
            decl.is_test(engines)
        } else {
            false
        }
    }
}
