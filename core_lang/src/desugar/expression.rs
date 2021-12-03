use crate::Expression;
use crate::error::{err, ok, CompileResult};

pub fn desugar_expression<'sc>(exp: Expression<'sc>) -> CompileResult<'sc, Expression<'sc>> {
    match exp {
        Expression::MatchExpression { primary_expression, branches, span } => {
            println!("{:#?}", branches);
            unimplemented!();
        },
        exp => unimplemented!("{:?}", exp)
    }
}