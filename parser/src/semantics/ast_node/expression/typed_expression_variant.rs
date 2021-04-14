use super::*;
use crate::semantics::ast_node::*;
use crate::Ident;
#[derive(Clone, Debug)]
pub(crate) enum TypedExpressionVariant<'sc> {
    Literal(Literal<'sc>),
    FunctionApplication {
        name: CallPath<'sc>,
        arguments: Vec<TypedExpression<'sc>>,
    },
    VariableExpression {
        unary_op: Option<UnaryOp>,
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
        asm: AsmExpression<'sc>,
    },
    // like a variable expression but it has multiple parts,
    // like looking up a field in a struct
    SubfieldExpression {
        unary_op: Option<UnaryOp>,
        name: Vec<Ident<'sc>>,
        span: Span<'sc>,
    },
    EnumInstantiation {
        /// for printing
        enum_name: Ident<'sc>,
        /// for printing
        variant_name: Ident<'sc>,
        tag: usize,
        contents: Option<Box<TypedExpression<'sc>>>,
    },
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
                    Literal::U128(content) => content.to_string(),
                    Literal::String(content) => content.to_string(),
                    Literal::Boolean(content) => content.to_string(),
                    Literal::Byte(content) => content.to_string(),
                    Literal::Byte32(content) => content
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                }
            ),
            TypedExpressionVariant::FunctionApplication { name, .. } => {
                format!("\"{}\" fn entry", name.suffix.primary_name)
            }
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
            TypedExpressionVariant::SubfieldExpression { span, .. } => {
                format!("\"{}\" subfield access", span.as_str())
            }
            TypedExpressionVariant::VariableExpression { name, .. } => {
                format!("\"{}\" variable exp", name.primary_name)
            }
            TypedExpressionVariant::EnumInstantiation {
                tag,
                enum_name,
                variant_name,
                ..
            } => {
                format!(
                    "{}::{} enum instantiation (tag: {})",
                    enum_name.primary_name, variant_name.primary_name, tag
                )
            }
        }
    }
}
