use crate::ast::Expression;
use crate::CodeBlock;
use either::Either;

#[derive(Debug)]
pub(crate) struct FunctionDeclaration<'sc> {
    name: &'sc str,
    body: CodeBlock<'sc>,
    parameters: Vec<FunctionParameter<'sc>>,
    span: pest::Span<'sc>,
}

#[derive(Debug)]
pub(crate) struct FunctionParameter<'sc> {
    name: &'sc str,
    r#type: TypeInfo,
}

/// Type information without an associated value, used for type inferencing and definition.
#[derive(Debug)]
enum TypeInfo {
    String,
    Integer,
    Boolean,
}
