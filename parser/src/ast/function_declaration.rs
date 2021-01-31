use crate::ast::Expression;
use crate::{CodeBlock, Rule};
use either::Either;
use pest::iterators::Pair;

#[derive(Debug)]
pub(crate) struct FunctionDeclaration<'sc> {
    pub(crate) name: &'sc str,
    pub(crate) body: CodeBlock<'sc>,
    pub(crate) parameters: Vec<FunctionParameter<'sc>>,
    pub(crate) span: pest::Span<'sc>,
}

#[derive(Debug)]
pub(crate) struct FunctionParameter<'sc> {
    name: &'sc str,
    r#type: TypeInfo,
}

impl<'sc> FunctionParameter<'sc> {
    pub(crate) fn list_from_pairs(
        pairs: impl Iterator<Item = Pair<'sc, Rule>>,
    ) -> Vec<FunctionParameter<'sc>> {
        pairs
            .map(|pair: Pair<'sc, Rule>| {
                println!(
                    "Unimplemented pair : {:?} ({:?})",
                    pair.as_str(),
                    pair.as_rule()
                );
                todo!("parse params!")
            })
            .collect()
    }
}

/// Type information without an associated value, used for type inferencing and definition.
#[derive(Debug)]
enum TypeInfo {
    String,
    Integer,
    Boolean,
}
