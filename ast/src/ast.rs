use parser::{HllParseTree, Span};

/// The initial typed syntax tree generated from source code.
pub(crate) struct Ast<'sc> {
    abi: Vec<Function<'sc>>,
}

pub(crate) struct Function<'sc> {
    visibility: FunctionVisibility,
    name: &'sc str,
    type_parameters: Vec<TypeParameter<'sc>>,
    parameters: Vec<FunctionParameter<'sc>>,
    body: Vec<Expression<'sc>>,
    return_type: Type<'sc>,
    span: Span<'sc>,
}

// TODO see issue #6 in HLL repo
enum FunctionVisibility {
    Public,
    Private,
}

struct Type<'sc> {
    todo: &'sc str,
}

pub struct TypeParameter<'sc> {
    todo: &'sc str,
}

pub struct FunctionParameter<'sc> {
    todo: &'sc str,
}

pub struct Expression<'sc> {
    todo: &'sc str,
}

impl<'sc> std::convert::From<HllParseTree<'sc>> for Ast<'sc> {
    fn from(ptree: HllParseTree<'sc>) -> Self {
        todo!()
    }
}
