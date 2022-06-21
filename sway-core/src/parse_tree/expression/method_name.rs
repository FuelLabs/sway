use std::fmt;

use sway_types::{Span, Spanned};

use crate::parse_tree::CallPath;
use crate::{Ident, TypeInfo};

#[derive(Debug, Clone)]
pub enum MethodName {
    /// Represents a method lookup with a type somewhere in the path
    /// something like blah::blah::~Type::foo()
    FromType {
        call_path: CallPath,
        type_name: TypeInfo,
        type_name_span: Span,
    },
    /// Represents a method lookup that does not contain any types in the path
    /// something like a.b(c)
    /// in this case, the first argument defines where to look for the method
    FromModule { method_name: Ident },
    /// something like a::b::c()
    /// in this case, the path defines where the fn symbol is defined
    /// used for things like core::ops::add(a, b).
    /// in this case, the first argument determines the type to look for
    FromTrait { call_path: CallPath },
}

impl fmt::Display for MethodName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            MethodName::FromType {
                call_path,
                type_name,
                ..
            } => {
                let mut builder = String::new();
                if call_path.is_absolute {
                    builder.push_str("::");
                }
                builder.push_str(&call_path.prefixes.iter().map(|x| x.to_string()).collect::<Vec<_>>().join("::"));
                builder.push_str(&format!("::~{}::{}", type_name, call_path.suffix));
                builder
            }
            MethodName::FromModule { method_name } => method_name.to_string(),
            MethodName::FromTrait { call_path } => call_path.to_string(),
        };
        write!(f, "{}", s)
    }
}

impl Spanned for MethodName {
    fn span(&self) -> Span {
        match self {
            MethodName::FromType { call_path, type_name, type_name_span } => {
                Span::join(call_path.span(), type_name_span.clone())
            },
            MethodName::FromModule { method_name } => method_name.span(),
            MethodName::FromTrait { call_path } => call_path.span(),
        }
    }
}

impl MethodName {
    /// To be used for error messages and debug strings
    pub fn easy_name(&self) -> Ident {
        match self {
            MethodName::FromType { call_path, .. } | MethodName::FromTrait { call_path, .. } => {
                call_path.suffix.clone()
            }
            MethodName::FromModule { method_name, .. } => method_name.clone(),
        }
    }
}
