use std::{cmp::Ordering, fmt, hash::Hasher};

use crate::{
    engine_threading::{
        DebugWithEngines, DisplayWithEngines, EqWithEngines, HashWithEngines, OrdWithEngines,
        PartialEqWithEngines,
    },
    language::{parsed::CodeBlock, *},
    type_system::TypeBinding,
    Engines, TypeArgs, TypeArgument, TypeId, TypeInfo,
};
use sway_error::handler::ErrorEmitted;
use sway_types::{ident::Ident, BaseIdent, Span, Spanned};

mod asm;
mod match_branch;
mod method_name;
mod scrutinee;
pub(crate) use asm::*;
pub(crate) use match_branch::MatchBranch;
pub use method_name::MethodName;
pub use scrutinee::*;
use sway_ast::intrinsics::Intrinsic;

/// Represents a parsed, but not yet type checked, [Expression](https://en.wikipedia.org/wiki/Expression_(computer_science)).
#[derive(Debug, Clone)]
pub struct Expression {
    pub kind: ExpressionKind,
    pub span: Span,
}

impl Expression {
    pub fn match_branch(value: Expression, branches: Vec<MatchBranch>) -> Self {
        Expression {
            kind: ExpressionKind::Match(MatchExpression {
                value: Box::new(value),
                branches,
            }),
            span: Span::dummy(),
        }
    }

    pub fn revert_u64(value: u64) -> Self {
        Expression {
            kind: ExpressionKind::IntrinsicFunction(IntrinsicFunctionExpression {
                name: Ident::new_no_span("__revert".into()),
                kind_binding: TypeBinding {
                    inner: Intrinsic::Revert,
                    type_arguments: TypeArgs::Regular(vec![]),
                    span: Span::dummy(),
                },
                arguments: vec![Expression {
                    kind: ExpressionKind::Literal(Literal::U64(value)),
                    span: Span::dummy(),
                }],
            }),
            span: Span::dummy(),
        }
    }

    pub fn literal_u64(value: u64) -> Self {
        Expression {
            kind: ExpressionKind::Literal(Literal::U64(value)),
            span: Span::dummy(),
        }
    }

    pub fn code_block(nodes: impl IntoIterator<Item = parsed::AstNode>) -> Self {
        Expression {
            kind: ExpressionKind::CodeBlock(CodeBlock {
                contents: nodes.into_iter().collect(),
                whole_block_span: Span::dummy(),
            }),
            span: Span::dummy(),
        }
    }

    pub fn ambiguous_variable_expression(name: BaseIdent) -> Self {
        Expression {
            kind: ExpressionKind::AmbiguousVariableExpression(name),
            span: Span::dummy(),
        }
    }

    pub fn call_function_with_suffix(suffix: BaseIdent, arguments: Vec<Expression>) -> Self {
        Expression {
            kind: ExpressionKind::FunctionApplication(Box::new(FunctionApplicationExpression {
                call_path_binding: TypeBinding {
                    inner: CallPath {
                        prefixes: vec![],
                        suffix,
                        is_absolute: false,
                    },
                    type_arguments: crate::TypeArgs::Regular(vec![]),
                    span: Span::dummy(),
                },
                arguments,
            })),
            span: Span::dummy(),
        }
    }

    pub fn call_method(method_name: BaseIdent, arguments: Vec<Expression>) -> Self {
        Expression {
            kind: ExpressionKind::MethodApplication(Box::new(MethodApplicationExpression {
                method_name_binding: TypeBinding {
                    inner: MethodName::FromModule { method_name },
                    type_arguments: TypeArgs::Regular(vec![]),
                    span: Span::dummy(),
                },
                contract_call_params: vec![],
                arguments,
            })),
            span: Span::dummy(),
        }
    }

    pub fn call_associated_function_as_trait(
        ty: TypeArgument,
        as_trait: TypeId,
        method_name: BaseIdent,
        arguments: Vec<Expression>,
    ) -> Self {
        Expression {
            kind: ExpressionKind::MethodApplication(Box::new(MethodApplicationExpression {
                method_name_binding: TypeBinding {
                    inner: MethodName::FromQualifiedPathRoot {
                        ty,
                        as_trait,
                        method_name,
                    },
                    type_arguments: TypeArgs::Regular(vec![]),
                    span: Span::dummy(),
                },
                contract_call_params: vec![],
                arguments,
            })),
            span: Span::dummy(),
        }
    }

    pub fn call_associated_function(
        suffix: (TypeInfo, BaseIdent),
        method_name: BaseIdent,
        arguments: Vec<Expression>,
    ) -> Self {
        Expression {
            kind: ExpressionKind::MethodApplication(Box::new(MethodApplicationExpression {
                method_name_binding: TypeBinding {
                    inner: MethodName::FromType {
                        call_path_binding: TypeBinding {
                            inner: CallPath {
                                prefixes: vec![],
                                suffix,
                                is_absolute: false,
                            },
                            type_arguments: TypeArgs::Regular(vec![]),
                            span: Span::dummy(),
                        },
                        method_name,
                    },
                    type_arguments: TypeArgs::Regular(vec![]),
                    span: Span::dummy(),
                },
                contract_call_params: vec![],
                arguments,
            })),
            span: Span::dummy(),
        }
    }

    pub fn retd(ptr: Expression, len: Expression) -> Self {
        Expression {
            kind: ExpressionKind::IntrinsicFunction(IntrinsicFunctionExpression {
                name: Ident::new_no_span("__contract_ret".into()),
                kind_binding: TypeBinding {
                    inner: Intrinsic::ContractRet,
                    type_arguments: TypeArgs::Regular(vec![]),
                    span: Span::dummy(),
                },
                arguments: vec![ptr, len],
            }),
            span: Span::dummy(),
        }
    }

    pub fn retd_addr_of_variable(var: BaseIdent, len: Expression) -> Self {
        Expression {
            kind: ExpressionKind::IntrinsicFunction(IntrinsicFunctionExpression {
                name: Ident::new_no_span("__contract_ret".into()),
                kind_binding: TypeBinding {
                    inner: Intrinsic::ContractRet,
                    type_arguments: TypeArgs::Regular(vec![]),
                    span: Span::dummy(),
                },
                arguments: vec![
                    Expression {
                        kind: ExpressionKind::IntrinsicFunction(IntrinsicFunctionExpression {
                            name: Ident::new_no_span("__addr_of".into()),
                            kind_binding: TypeBinding {
                                inner: Intrinsic::AddrOf,
                                type_arguments: TypeArgs::Regular(vec![]),
                                span: Span::dummy(),
                            },
                            arguments: vec![Expression::ambiguous_variable_expression(var)],
                        }),
                        span: Span::dummy(),
                    },
                    len,
                ],
            }),
            span: Span::dummy(),
        }
    }

    pub fn subfield(prefix: Expression, field_to_access: BaseIdent) -> Self {
        Expression {
            kind: ExpressionKind::Subfield(SubfieldExpression {
                prefix: Box::new(prefix),
                field_to_access,
            }),
            span: Span::dummy(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FunctionApplicationExpression {
    pub call_path_binding: TypeBinding<CallPath>,
    pub arguments: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub struct LazyOperatorExpression {
    pub op: LazyOp,
    pub lhs: Box<Expression>,
    pub rhs: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct TupleIndexExpression {
    pub prefix: Box<Expression>,
    pub index: usize,
    pub index_span: Span,
}

#[derive(Debug, Clone)]
pub struct ArrayExpression {
    pub contents: Vec<Expression>,
    pub length_span: Option<Span>,
}

#[derive(Debug, Clone)]
pub struct StructExpression {
    pub call_path_binding: TypeBinding<CallPath>,
    pub fields: Vec<StructExpressionField>,
}

#[derive(Debug, Clone)]
pub struct IfExpression {
    pub condition: Box<Expression>,
    pub then: Box<Expression>,
    pub r#else: Option<Box<Expression>>,
}

#[derive(Debug, Clone)]
pub struct MatchExpression {
    pub value: Box<Expression>,
    pub branches: Vec<MatchBranch>,
}

#[derive(Debug, Clone)]
pub struct MethodApplicationExpression {
    pub method_name_binding: TypeBinding<MethodName>,
    pub contract_call_params: Vec<StructExpressionField>,
    pub arguments: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub struct SubfieldExpression {
    pub prefix: Box<Expression>,
    pub field_to_access: Ident,
}

#[derive(Debug, Clone)]
pub struct AmbiguousSuffix {
    /// If the suffix is a pair, the ambiguous part of the suffix.
    ///
    /// For example, if we have `Foo::bar()`,
    /// we don't know whether `Foo` is a module or a type,
    /// so `before` would be `Foo` here with any type arguments.
    pub before: Option<TypeBinding<Ident>>,
    /// The final suffix, i.e., the function or variant name.
    ///
    /// In the example above, this would be `bar`.
    pub suffix: Ident,
}

impl Spanned for AmbiguousSuffix {
    fn span(&self) -> Span {
        if let Some(before) = &self.before {
            Span::join(before.span(), self.suffix.span())
        } else {
            self.suffix.span()
        }
    }
}

#[derive(Debug, Clone)]
pub struct QualifiedPathRootTypes {
    pub ty: TypeArgument,
    pub as_trait: TypeId,
    pub as_trait_span: Span,
}

impl HashWithEngines for QualifiedPathRootTypes {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        let QualifiedPathRootTypes {
            ty,
            as_trait,
            // ignored fields
            as_trait_span: _,
        } = self;
        ty.hash(state, engines);
        engines.te().get(*as_trait).hash(state, engines);
    }
}

impl EqWithEngines for QualifiedPathRootTypes {}
impl PartialEqWithEngines for QualifiedPathRootTypes {
    fn eq(&self, other: &Self, engines: &Engines) -> bool {
        let QualifiedPathRootTypes {
            ty,
            as_trait,
            // ignored fields
            as_trait_span: _,
        } = self;
        ty.eq(&other.ty, engines)
            && engines
                .te()
                .get(*as_trait)
                .eq(&engines.te().get(other.as_trait), engines)
    }
}

impl OrdWithEngines for QualifiedPathRootTypes {
    fn cmp(&self, other: &Self, engines: &Engines) -> Ordering {
        let QualifiedPathRootTypes {
            ty: l_ty,
            as_trait: l_as_trait,
            // ignored fields
            as_trait_span: _,
        } = self;
        let QualifiedPathRootTypes {
            ty: r_ty,
            as_trait: r_as_trait,
            // ignored fields
            as_trait_span: _,
        } = other;
        l_ty.cmp(r_ty, engines).then_with(|| {
            engines
                .te()
                .get(*l_as_trait)
                .cmp(&engines.te().get(*r_as_trait), engines)
        })
    }
}

impl DisplayWithEngines for QualifiedPathRootTypes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        write!(
            f,
            "<{} as {}>",
            engines.help_out(self.ty.clone()),
            engines.help_out(self.as_trait)
        )
    }
}

impl DebugWithEngines for QualifiedPathRootTypes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        write!(f, "{}", engines.help_out(self),)
    }
}

#[derive(Debug, Clone)]
pub struct AmbiguousPathExpression {
    pub qualified_path_root: Option<QualifiedPathRootTypes>,
    pub call_path_binding: TypeBinding<CallPath<AmbiguousSuffix>>,
    pub args: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub struct DelineatedPathExpression {
    pub call_path_binding: TypeBinding<QualifiedCallPath>,
    /// When args is equal to Option::None then it means that the
    /// [DelineatedPathExpression] was initialized from an expression
    /// that does not end with parenthesis.
    pub args: Option<Vec<Expression>>,
}

#[derive(Debug, Clone)]
pub struct AbiCastExpression {
    pub abi_name: CallPath,
    pub address: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct ArrayIndexExpression {
    pub prefix: Box<Expression>,
    pub index: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct StorageAccessExpression {
    pub field_names: Vec<Ident>,
    pub storage_keyword_span: Span,
}

#[derive(Debug, Clone)]
pub struct IntrinsicFunctionExpression {
    pub name: Ident,
    pub kind_binding: TypeBinding<Intrinsic>,
    pub arguments: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub struct WhileLoopExpression {
    pub condition: Box<Expression>,
    pub body: CodeBlock,
}

#[derive(Debug, Clone)]
pub struct ForLoopExpression {
    pub desugared: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct ReassignmentExpression {
    pub lhs: ReassignmentTarget,
    pub rhs: Box<Expression>,
}

#[derive(Debug, Clone)]
pub enum ExpressionKind {
    /// A malformed expression.
    ///
    /// Used for parser recovery when we cannot form a more specific node.
    /// The list of `Span`s are for consumption by the LSP and are,
    /// when joined, the same as that stored in `expr.span`.
    Error(Box<[Span]>, ErrorEmitted),
    Literal(Literal),
    /// An ambiguous path where we don't know until type checking whether this
    /// is a free function call, an enum variant or a UFCS (Rust term) style associated function call.
    AmbiguousPathExpression(Box<AmbiguousPathExpression>),
    FunctionApplication(Box<FunctionApplicationExpression>),
    LazyOperator(LazyOperatorExpression),
    /// And ambiguous single ident which could either be a variable or an enum variant
    AmbiguousVariableExpression(Ident),
    Variable(Ident),
    Tuple(Vec<Expression>),
    TupleIndex(TupleIndexExpression),
    Array(ArrayExpression),
    Struct(Box<StructExpression>),
    CodeBlock(CodeBlock),
    If(IfExpression),
    Match(MatchExpression),
    // separated into other struct for parsing reasons
    Asm(Box<AsmExpression>),
    MethodApplication(Box<MethodApplicationExpression>),
    /// A _subfield expression_ is anything of the form:
    /// ```ignore
    /// <ident>.<ident>
    /// ```
    ///
    Subfield(SubfieldExpression),
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
    DelineatedPath(Box<DelineatedPathExpression>),
    /// A cast of a hash to an ABI for calling a contract.
    AbiCast(Box<AbiCastExpression>),
    ArrayIndex(ArrayIndexExpression),
    StorageAccess(StorageAccessExpression),
    IntrinsicFunction(IntrinsicFunctionExpression),
    /// A control flow element which loops continually until some boolean expression evaluates as
    /// `false`.
    WhileLoop(WhileLoopExpression),
    /// A control flow element which loops between values of an iterator.
    ForLoop(ForLoopExpression),
    Break,
    Continue,
    Reassignment(ReassignmentExpression),
    /// An implicit return expression is different from a [Expression::Return] because
    /// it is not a control flow item. Therefore it is a different variant.
    ///
    /// An implicit return expression is an [Expression] at the end of a code block which has no
    /// semicolon, denoting that it is the [Expression] to be returned from that block.
    ImplicitReturn(Box<Expression>),
    Return(Box<Expression>),
    Ref(RefExpression),
    Deref(Box<Expression>),
}

#[derive(Debug, Clone)]
pub struct RefExpression {
    /// True if the reference is a reference to a mutable `value`.
    pub to_mutable_value: bool,
    pub value: Box<Expression>,
}

#[derive(Debug, Clone)]
pub enum ReassignmentTarget {
    VariableExpression(Box<Expression>),
}

#[derive(Debug, Clone)]
pub struct StructExpressionField {
    pub name: Ident,
    pub value: Expression,
}

impl Spanned for Expression {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

#[derive(Debug)]
pub(crate) struct Op {
    pub span: Span,
    pub op_variant: OpVariant,
}

impl Op {
    pub fn to_var_name(&self) -> Ident {
        Ident::new_with_override(self.op_variant.as_str().to_string(), self.span.clone())
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
