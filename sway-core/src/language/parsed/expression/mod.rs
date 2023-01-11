use crate::{
    language::{parsed::CodeBlock, *},
    type_system::TypeBinding,
};
use sway_types::{ident::Ident, Span, Spanned};

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
    /// The ambiguous part of the suffix.
    ///
    /// For example, if we have `Foo::bar()`,
    /// we don't know whether `Foo` is a module or a type,
    /// so `before` would be `Foo` here with any type arguments.
    pub before: TypeBinding<Ident>,
    /// The final suffix, i.e., the function name.
    ///
    /// In the example above, this would be `bar`.
    pub suffix: Ident,
}

impl Spanned for AmbiguousSuffix {
    fn span(&self) -> Span {
        Span::join(self.before.span(), self.suffix.span())
    }
}

#[derive(Debug, Clone)]
pub struct AmbiguousPathExpression {
    pub call_path_binding: TypeBinding<CallPath<AmbiguousSuffix>>,
    pub args: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub struct DelineatedPathExpression {
    pub call_path_binding: TypeBinding<CallPath>,
    pub args: Vec<Expression>,
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
    Error(Box<[Span]>),
    Literal(Literal),
    /// An ambiguous path where we don't know until type checking whether this
    /// is a free function call or a UFCS (Rust term) style associated function call.
    AmbiguousPathExpression(Box<AmbiguousPathExpression>),
    FunctionApplication(Box<FunctionApplicationExpression>),
    LazyOperator(LazyOperatorExpression),
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
    Break,
    Continue,
    Reassignment(ReassignmentExpression),
    Return(Box<Expression>),
}

/// Represents the left hand side of a reassignment, which could either be a regular variable
/// expression, denoted by [ReassignmentTarget::VariableExpression], or, a storage field, denoted
/// by [ReassignmentTarget::StorageField].
#[derive(Debug, Clone)]
pub enum ReassignmentTarget {
    VariableExpression(Box<Expression>),
    StorageField(Vec<Ident>),
}

#[derive(Debug, Clone)]
pub struct StructExpressionField {
    pub name: Ident,
    pub value: Expression,
    pub(crate) span: Span,
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
