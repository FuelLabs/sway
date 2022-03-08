use sway_core::{Literal, VariableDeclaration, Visibility};

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
        Expression::Literal { value, .. } => match value {
            Literal::U8(_) => VarBody::Type("u8".into()),
            Literal::U16(_) => VarBody::Type("u16".into()),
            Literal::U32(_) => VarBody::Type("u32".into()),
            Literal::U64(_) => VarBody::Type("u64".into()),
            Literal::Numeric(_) => VarBody::Type("u64".into()),
            Literal::String(len) => VarBody::Type(format!("str[{}]", len.as_str().len())),
            Literal::Boolean(_) => VarBody::Type("bool".into()),
            Literal::Byte(_) => VarBody::Type("u8".into()),
            Literal::B256(_) => VarBody::Type("b256".into()),
        },
        _ => VarBody::Other,
    }
}
