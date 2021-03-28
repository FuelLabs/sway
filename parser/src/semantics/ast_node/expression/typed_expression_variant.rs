use super::*;
use crate::parse_tree::Ident;
use crate::semantics::ast_node::*;
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
}
