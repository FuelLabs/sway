use crate::{
    decl_engine::parsed_id::ParsedDeclId,
    engine_threading::{
        DebugWithEngines, DisplayWithEngines, EqWithEngines, HashWithEngines, OrdWithEngines,
        OrdWithEnginesContext, PartialEqWithEngines, PartialEqWithEnginesContext,
    },
    language::{parsed::CodeBlock, *},
    type_system::TypeBinding,
    Engines, GenericArgument, TypeId,
};
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, fmt, hash::Hasher};
use sway_error::handler::ErrorEmitted;
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

use super::{FunctionDeclaration, StructDeclaration};

/// Represents a parsed, but not yet type checked, [Expression](https://en.wikipedia.org/wiki/Expression_(computer_science)).
#[derive(Debug, Clone)]
pub struct Expression {
    pub kind: ExpressionKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct FunctionApplicationExpression {
    pub call_path_binding: TypeBinding<CallPath>,
    pub resolved_call_path_binding:
        Option<TypeBinding<ResolvedCallPath<ParsedDeclId<FunctionDeclaration>>>>,
    pub arguments: Vec<Expression>,
}

impl EqWithEngines for FunctionApplicationExpression {}
impl PartialEqWithEngines for FunctionApplicationExpression {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.call_path_binding.eq(&other.call_path_binding, ctx)
            && self.arguments.eq(&other.arguments, ctx)
    }
}

#[derive(Debug, Clone)]
pub struct LazyOperatorExpression {
    pub op: LazyOp,
    pub lhs: Box<Expression>,
    pub rhs: Box<Expression>,
}

impl EqWithEngines for LazyOperatorExpression {}
impl PartialEqWithEngines for LazyOperatorExpression {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.op == other.op && self.lhs.eq(&other.lhs, ctx) && self.rhs.eq(&other.rhs, ctx)
    }
}

#[derive(Debug, Clone)]
pub struct TupleIndexExpression {
    pub prefix: Box<Expression>,
    pub index: usize,
    pub index_span: Span,
}

impl EqWithEngines for TupleIndexExpression {}
impl PartialEqWithEngines for TupleIndexExpression {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.prefix.eq(&other.prefix, ctx)
            && self.index == other.index
            && self.index_span == other.index_span
    }
}

#[derive(Debug, Clone)]
pub enum ArrayExpression {
    Explicit {
        contents: Vec<Expression>,
        length_span: Option<Span>,
    },
    Repeat {
        value: Box<Expression>,
        length: Box<Expression>,
    },
}

impl EqWithEngines for ArrayExpression {}
impl PartialEqWithEngines for ArrayExpression {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        match (self, other) {
            (
                ArrayExpression::Explicit {
                    contents: self_contents,
                    length_span: self_length_span,
                },
                ArrayExpression::Explicit {
                    contents: other_contents,
                    length_span: other_length_span,
                },
            ) => self_contents.eq(other_contents, ctx) && self_length_span == other_length_span,
            (
                ArrayExpression::Repeat {
                    value: self_value,
                    length: self_length,
                },
                ArrayExpression::Repeat {
                    value: other_value,
                    length: other_length,
                },
            ) => self_value.eq(other_value, ctx) && self_length.eq(other_length, ctx),
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StructExpression {
    pub resolved_call_path_binding:
        Option<TypeBinding<ResolvedCallPath<ParsedDeclId<StructDeclaration>>>>,
    pub call_path_binding: TypeBinding<CallPath>,
    pub fields: Vec<StructExpressionField>,
}

impl EqWithEngines for StructExpression {}
impl PartialEqWithEngines for StructExpression {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.call_path_binding.eq(&other.call_path_binding, ctx)
            && self.fields.eq(&other.fields, ctx)
    }
}

#[derive(Debug, Clone)]
pub struct IfExpression {
    pub condition: Box<Expression>,
    pub then: Box<Expression>,
    pub r#else: Option<Box<Expression>>,
}

impl EqWithEngines for IfExpression {}
impl PartialEqWithEngines for IfExpression {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.condition.eq(&other.condition, ctx)
            && self.then.eq(&other.then, ctx)
            && self.r#else.eq(&other.r#else, ctx)
    }
}

#[derive(Debug, Clone)]
pub struct MatchExpression {
    pub value: Box<Expression>,
    pub branches: Vec<MatchBranch>,
}

impl EqWithEngines for MatchExpression {}
impl PartialEqWithEngines for MatchExpression {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.value.eq(&other.value, ctx) && self.branches.eq(&other.branches, ctx)
    }
}

#[derive(Debug, Clone)]
pub struct MethodApplicationExpression {
    pub method_name_binding: TypeBinding<MethodName>,
    pub contract_call_params: Vec<StructExpressionField>,
    pub arguments: Vec<Expression>,
}

impl EqWithEngines for MethodApplicationExpression {}
impl PartialEqWithEngines for MethodApplicationExpression {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.method_name_binding.eq(&other.method_name_binding, ctx)
            && self
                .contract_call_params
                .eq(&other.contract_call_params, ctx)
            && self.arguments.eq(&other.arguments, ctx)
    }
}

#[derive(Debug, Clone)]
pub struct SubfieldExpression {
    pub prefix: Box<Expression>,
    pub field_to_access: Ident,
}

impl EqWithEngines for SubfieldExpression {}
impl PartialEqWithEngines for SubfieldExpression {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.prefix.eq(&other.prefix, ctx) && self.field_to_access == other.field_to_access
    }
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

impl EqWithEngines for AmbiguousSuffix {}
impl PartialEqWithEngines for AmbiguousSuffix {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.before.eq(&other.before, ctx) && self.suffix == other.suffix
    }
}

impl Spanned for AmbiguousSuffix {
    fn span(&self) -> Span {
        if let Some(before) = &self.before {
            Span::join(before.span(), &self.suffix.span())
        } else {
            self.suffix.span()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualifiedPathType {
    pub ty: GenericArgument,
    pub as_trait: TypeId,
    pub as_trait_span: Span,
}

impl HashWithEngines for QualifiedPathType {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        let QualifiedPathType {
            ty,
            as_trait,
            // ignored fields
            as_trait_span: _,
        } = self;
        ty.hash(state, engines);
        engines.te().get(*as_trait).hash(state, engines);
    }
}

impl EqWithEngines for QualifiedPathType {}
impl PartialEqWithEngines for QualifiedPathType {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        let QualifiedPathType {
            ty,
            as_trait,
            // ignored fields
            as_trait_span: _,
        } = self;
        ty.eq(&other.ty, ctx)
            && ctx
                .engines()
                .te()
                .get(*as_trait)
                .eq(&ctx.engines().te().get(other.as_trait), ctx)
    }
}

impl OrdWithEngines for QualifiedPathType {
    fn cmp(&self, other: &Self, ctx: &OrdWithEnginesContext) -> Ordering {
        let QualifiedPathType {
            ty: l_ty,
            as_trait: l_as_trait,
            // ignored fields
            as_trait_span: _,
        } = self;
        let QualifiedPathType {
            ty: r_ty,
            as_trait: r_as_trait,
            // ignored fields
            as_trait_span: _,
        } = other;
        l_ty.cmp(r_ty, ctx).then_with(|| {
            ctx.engines()
                .te()
                .get(*l_as_trait)
                .cmp(&ctx.engines().te().get(*r_as_trait), ctx)
        })
    }
}

impl DisplayWithEngines for QualifiedPathType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        write!(
            f,
            "<{} as {}>",
            engines.help_out(self.ty.clone()),
            engines.help_out(self.as_trait)
        )
    }
}

impl DebugWithEngines for QualifiedPathType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        write!(f, "{}", engines.help_out(self),)
    }
}

#[derive(Debug, Clone)]
pub struct AmbiguousPathExpression {
    pub qualified_path_root: Option<QualifiedPathType>,
    pub call_path_binding: TypeBinding<CallPath<AmbiguousSuffix>>,
    pub args: Vec<Expression>,
}

impl EqWithEngines for AmbiguousPathExpression {}
impl PartialEqWithEngines for AmbiguousPathExpression {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.qualified_path_root.eq(&other.qualified_path_root, ctx)
            && PartialEqWithEngines::eq(&self.call_path_binding, &other.call_path_binding, ctx)
            && self.args.eq(&other.args, ctx)
    }
}

#[derive(Debug, Clone)]
pub struct DelineatedPathExpression {
    pub call_path_binding: TypeBinding<QualifiedCallPath>,
    /// When args is equal to Option::None then it means that the
    /// [DelineatedPathExpression] was initialized from an expression
    /// that does not end with parenthesis.
    pub args: Option<Vec<Expression>>,
}

impl EqWithEngines for DelineatedPathExpression {}
impl PartialEqWithEngines for DelineatedPathExpression {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.call_path_binding.eq(&other.call_path_binding, ctx) && self.args.eq(&other.args, ctx)
    }
}

#[derive(Debug, Clone)]
pub struct AbiCastExpression {
    pub abi_name: CallPath,
    pub address: Box<Expression>,
}

impl EqWithEngines for AbiCastExpression {}
impl PartialEqWithEngines for AbiCastExpression {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        PartialEqWithEngines::eq(&self.abi_name, &other.abi_name, ctx)
            && self.address.eq(&other.address, ctx)
    }
}

#[derive(Debug, Clone)]
pub struct ArrayIndexExpression {
    pub prefix: Box<Expression>,
    pub index: Box<Expression>,
}

impl EqWithEngines for ArrayIndexExpression {}
impl PartialEqWithEngines for ArrayIndexExpression {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.prefix.eq(&other.prefix, ctx) && self.index.eq(&other.index, ctx)
    }
}

#[derive(Debug, Clone)]
pub struct StorageAccessExpression {
    pub namespace_names: Vec<Ident>,
    pub field_names: Vec<Ident>,
    pub storage_keyword_span: Span,
}

impl EqWithEngines for StorageAccessExpression {}
impl PartialEqWithEngines for StorageAccessExpression {
    fn eq(&self, other: &Self, _ctx: &PartialEqWithEnginesContext) -> bool {
        self.field_names.eq(&other.field_names)
            && self.storage_keyword_span.eq(&other.storage_keyword_span)
    }
}

#[derive(Debug, Clone)]
pub struct IntrinsicFunctionExpression {
    pub name: Ident,
    pub kind_binding: TypeBinding<Intrinsic>,
    pub arguments: Vec<Expression>,
}

impl EqWithEngines for IntrinsicFunctionExpression {}
impl PartialEqWithEngines for IntrinsicFunctionExpression {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.name.eq(&other.name)
            && self.kind_binding.eq(&other.kind_binding, ctx)
            && self.arguments.eq(&other.arguments, ctx)
    }
}

#[derive(Debug, Clone)]
pub struct WhileLoopExpression {
    pub condition: Box<Expression>,
    pub body: CodeBlock,
    pub is_desugared_for_loop: bool,
}

impl EqWithEngines for WhileLoopExpression {}
impl PartialEqWithEngines for WhileLoopExpression {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.condition.eq(&other.condition, ctx) && self.body.eq(&other.body, ctx)
    }
}

#[derive(Debug, Clone)]
pub struct ForLoopExpression {
    pub desugared: Box<Expression>,
}

impl EqWithEngines for ForLoopExpression {}
impl PartialEqWithEngines for ForLoopExpression {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.desugared.eq(&other.desugared, ctx)
    }
}

#[derive(Debug, Clone)]
pub struct ReassignmentExpression {
    pub lhs: ReassignmentTarget,
    pub rhs: Box<Expression>,
}

impl EqWithEngines for ReassignmentExpression {}
impl PartialEqWithEngines for ReassignmentExpression {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.lhs.eq(&other.lhs, ctx) && self.rhs.eq(&other.rhs, ctx)
    }
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
    Panic(Box<Expression>),
    Ref(RefExpression),
    Deref(Box<Expression>),
}

impl EqWithEngines for Expression {}
impl PartialEqWithEngines for Expression {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.kind.eq(&other.kind, ctx)
    }
}

impl EqWithEngines for ExpressionKind {}
impl PartialEqWithEngines for ExpressionKind {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        match (self, other) {
            (ExpressionKind::Error(l_span, _), ExpressionKind::Error(r_span, _)) => {
                l_span == r_span
            }
            (ExpressionKind::Literal(l_literal), ExpressionKind::Literal(r_literal)) => {
                l_literal == r_literal
            }
            (
                ExpressionKind::AmbiguousPathExpression(lhs),
                ExpressionKind::AmbiguousPathExpression(rhs),
            ) => lhs.eq(rhs, ctx),
            (
                ExpressionKind::FunctionApplication(lhs),
                ExpressionKind::FunctionApplication(rhs),
            ) => lhs.eq(rhs, ctx),
            (ExpressionKind::LazyOperator(lhs), ExpressionKind::LazyOperator(rhs)) => {
                lhs.eq(rhs, ctx)
            }
            (
                ExpressionKind::AmbiguousVariableExpression(lhs),
                ExpressionKind::AmbiguousVariableExpression(rhs),
            ) => lhs == rhs,
            (ExpressionKind::Variable(lhs), ExpressionKind::Variable(rhs)) => lhs == rhs,
            (ExpressionKind::Tuple(lhs), ExpressionKind::Tuple(rhs)) => lhs.eq(rhs, ctx),
            (ExpressionKind::TupleIndex(lhs), ExpressionKind::TupleIndex(rhs)) => lhs.eq(rhs, ctx),
            (ExpressionKind::Array(lhs), ExpressionKind::Array(rhs)) => lhs.eq(rhs, ctx),
            (ExpressionKind::Struct(lhs), ExpressionKind::Struct(rhs)) => lhs.eq(rhs, ctx),
            (ExpressionKind::CodeBlock(lhs), ExpressionKind::CodeBlock(rhs)) => lhs.eq(rhs, ctx),
            (ExpressionKind::If(lhs), ExpressionKind::If(rhs)) => lhs.eq(rhs, ctx),
            (ExpressionKind::Match(lhs), ExpressionKind::Match(rhs)) => lhs.eq(rhs, ctx),
            (ExpressionKind::Asm(lhs), ExpressionKind::Asm(rhs)) => lhs.eq(rhs, ctx),
            (ExpressionKind::MethodApplication(lhs), ExpressionKind::MethodApplication(rhs)) => {
                lhs.eq(rhs, ctx)
            }
            (ExpressionKind::Subfield(lhs), ExpressionKind::Subfield(rhs)) => lhs.eq(rhs, ctx),
            (ExpressionKind::DelineatedPath(lhs), ExpressionKind::DelineatedPath(rhs)) => {
                lhs.eq(rhs, ctx)
            }
            (ExpressionKind::AbiCast(lhs), ExpressionKind::AbiCast(rhs)) => lhs.eq(rhs, ctx),
            (ExpressionKind::ArrayIndex(lhs), ExpressionKind::ArrayIndex(rhs)) => lhs.eq(rhs, ctx),
            (ExpressionKind::StorageAccess(lhs), ExpressionKind::StorageAccess(rhs)) => {
                lhs.eq(rhs, ctx)
            }
            (ExpressionKind::IntrinsicFunction(lhs), ExpressionKind::IntrinsicFunction(rhs)) => {
                lhs.eq(rhs, ctx)
            }
            (ExpressionKind::WhileLoop(lhs), ExpressionKind::WhileLoop(rhs)) => lhs.eq(rhs, ctx),
            (ExpressionKind::ForLoop(lhs), ExpressionKind::ForLoop(rhs)) => lhs.eq(rhs, ctx),
            (ExpressionKind::Break, ExpressionKind::Break) => true,
            (ExpressionKind::Continue, ExpressionKind::Continue) => true,
            (ExpressionKind::Reassignment(lhs), ExpressionKind::Reassignment(rhs)) => {
                lhs.eq(rhs, ctx)
            }
            (ExpressionKind::ImplicitReturn(lhs), ExpressionKind::ImplicitReturn(rhs)) => {
                lhs.eq(rhs, ctx)
            }
            (ExpressionKind::Return(lhs), ExpressionKind::Return(rhs)) => lhs.eq(rhs, ctx),
            (ExpressionKind::Panic(lhs), ExpressionKind::Panic(rhs)) => lhs.eq(rhs, ctx),
            (ExpressionKind::Ref(lhs), ExpressionKind::Ref(rhs)) => lhs.eq(rhs, ctx),
            (ExpressionKind::Deref(lhs), ExpressionKind::Deref(rhs)) => lhs.eq(rhs, ctx),
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RefExpression {
    /// True if the reference is a reference to a mutable `value`.
    pub to_mutable_value: bool,
    pub value: Box<Expression>,
}

impl EqWithEngines for RefExpression {}
impl PartialEqWithEngines for RefExpression {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.to_mutable_value.eq(&other.to_mutable_value) && self.value.eq(&other.value, ctx)
    }
}

#[derive(Debug, Clone)]
pub enum ReassignmentTarget {
    /// An [Expression] representing a single variable or a path
    /// to a part of an aggregate.
    /// E.g.:
    ///  - `my_variable`
    ///  - `array[0].field.x.1`
    ElementAccess(Box<Expression>),
    /// An dereferencing [Expression] representing dereferencing
    /// of an arbitrary reference expression.
    /// E.g.:
    ///  - *my_ref
    ///  - **if x > 0 { &mut &mut a } else { &mut &mut b }
    Deref(Box<Expression>),
}

impl EqWithEngines for ReassignmentTarget {}
impl PartialEqWithEngines for ReassignmentTarget {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        match (self, other) {
            (ReassignmentTarget::ElementAccess(lhs), ReassignmentTarget::ElementAccess(rhs)) => {
                lhs.eq(rhs, ctx)
            }
            (ReassignmentTarget::Deref(lhs), ReassignmentTarget::Deref(rhs)) => lhs.eq(rhs, ctx),
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StructExpressionField {
    pub name: Ident,
    pub value: Expression,
}

impl EqWithEngines for StructExpressionField {}
impl PartialEqWithEngines for StructExpressionField {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.name == other.name && self.value.eq(&other.value, ctx)
    }
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
    pub fn to_method_name(&self) -> Ident {
        Ident::new_with_override(self.op_variant.method_name().to_string(), self.span.clone())
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
    /// For all the operators except [OpVariant::Or] and [OpVariant::And],
    /// returns the name of the method that can be found on the corresponding
    /// operator trait. E.g., for `+` that will be the method `add` defined in
    /// `std::ops::Add::add`.
    ///
    /// [OpVariant::Or] and [OpVariant::And] are lazy and must be handled
    /// internally by the compiler.
    fn method_name(&self) -> &'static str {
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
