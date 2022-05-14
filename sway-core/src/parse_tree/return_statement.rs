use crate::{
    build_config::BuildConfig,
    error::{ok, ParserLifter},
    error_recovery_exp, CompileResult, Expression,
};

use sway_types::span;

#[derive(Debug, Clone)]
pub struct ReturnStatement {
    pub expr: Expression,
}
