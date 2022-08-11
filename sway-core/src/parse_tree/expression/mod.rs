use crate::{
    parse_tree::{CallPath, Literal},
    type_system::TypeBinding,
    CodeBlock, TypeInfo,
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
pub enum Expression {
    Literal {
        value: Literal,
        span: Span,
    },
    FunctionApplication {
        call_path_binding: TypeBinding<CallPath>,
        arguments: Vec<Expression>,
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
        call_path_binding: TypeBinding<CallPath<(TypeInfo, Span)>>,
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
        value: Box<Expression>,
        branches: Vec<MatchBranch>,
        span: Span,
    },
    // separated into other struct for parsing reasons
    AsmExpression {
        span: Span,
        asm: AsmExpression,
    },
    MethodApplication {
        method_name_binding: TypeBinding<MethodName>,
        contract_call_params: Vec<StructExpressionField>,
        arguments: Vec<Expression>,
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
        call_path_binding: TypeBinding<CallPath>,
        args: Vec<Expression>,
        span: Span,
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
    StorageAccess {
        field_names: Vec<Ident>,
        span: Span,
    },
    IntrinsicFunction {
        kind_binding: TypeBinding<Intrinsic>,
        arguments: Vec<Expression>,
        span: Span,
    },
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

impl Spanned for Expression {
    fn span(&self) -> Span {
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
            StorageAccess { span, .. } => span,
            IntrinsicFunction { span, .. } => span,
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
