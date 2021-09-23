use super::*;
use crate::parse_tree::AsmOp;
use crate::semantic_analysis::ast_node::*;
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
    #[allow(dead_code)]
    Array {
        contents: Vec<TypedExpression<'sc>>,
    },
    #[allow(dead_code)]
    MatchExpression {
        primary_expression: Box<TypedExpression<'sc>>,
        branches: Vec<TypedMatchBranch<'sc>>,
    },
    StructExpression {
        struct_name: Ident<'sc>,
        fields: Vec<TypedStructExpressionField<'sc>>,
    },
    CodeBlock(TypedCodeBlock<'sc>),
    // a flag that this value will later be provided as a parameter, but is currently unknown
    FunctionParameter,
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
        field_to_access: TypedStructField<'sc>,
        resolved_type_of_parent: MaybeResolvedType<'sc>,
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
        span: Span<'sc>,
        abi: TypedAbiDeclaration<'sc>,
    },
}

#[derive(Clone, Debug)]
pub(crate) struct TypedAsmRegisterDeclaration<'sc> {
    pub(crate) initializer: Option<TypedExpression<'sc>>,
    pub(crate) name: &'sc str,
    pub(crate) name_span: Span<'sc>,
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
            TypedExpressionVariant::MatchExpression { .. } => "match exp".into(),
            TypedExpressionVariant::StructExpression { struct_name, .. } => {
                format!("\"{}\" struct init", struct_name.primary_name)
            }
            TypedExpressionVariant::CodeBlock(_) => "code block entry".into(),
            TypedExpressionVariant::FunctionParameter => "fn param access".into(),
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
                    resolved_type_of_parent.friendly_type_str(),
                    field_to_access.name.primary_name
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
        }
    }
}
