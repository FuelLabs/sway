use crate::Expression;
use crate::error::{err, ok, CompileResult};

pub fn desugar_expression<'sc>(exp: Expression<'sc>) -> CompileResult<'sc, Expression<'sc>> {
    unimplemented!()
}