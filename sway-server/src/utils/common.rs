use sway_core::{VariableDeclaration, Visibility};

use crate::core::token_type::VarBody;
use sway_core::Expression;

pub(crate) fn extract_visibility(visibility: &Visibility) -> String {
    match visibility {
        Visibility::Private => "".into(),
        Visibility::Public => "pub ".into(),
    }
}

pub(crate) fn extract_var_body(var_dec: &VariableDeclaration) -> VarBody {
    match &var_dec.body {
        Expression::FunctionApplication { name, .. } => {
            VarBody::FunctionCall(name.suffix.as_str().into())
        }
        Expression::StructExpression { struct_name, .. } => {
            VarBody::Type(struct_name.suffix.as_str().into())
        }
        _ => VarBody::Other,
    }
}
