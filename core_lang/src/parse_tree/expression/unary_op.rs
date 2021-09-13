use crate::error::*;
use crate::parse_tree::Expression;
use crate::parser::Rule;
use pest::iterators::Pair;

#[derive(Clone, Debug)]
pub enum UnaryOp {
    Not,
    Ref,
    Deref,
}

impl UnaryOp {
    pub fn parse_from_pair<'sc>(pair: Pair<'sc, Rule>) -> CompileResult<'sc, Self> {
        use UnaryOp::*;
        match pair.as_str() {
            "!" => ok(Not, Vec::new(), Vec::new()),
            "ref" => ok(Ref, Vec::new(), Vec::new()),
            "deref" => ok(Deref, Vec::new(), Vec::new()),
            _ => {
                let errors = vec![CompileError::Internal(
                    "Attempted to parse unary op from invalid op string.",
                    pair.as_span(),
                )];
                return err(Vec::new(), errors);
            }
        }
    }

    pub fn to_fn_application<'sc>(&self, arg: Expression<'sc>) -> Expression<'sc> {
        todo!()
    }
}
