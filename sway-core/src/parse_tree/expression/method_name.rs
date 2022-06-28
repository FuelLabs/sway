use sway_types::Span;

use crate::parse_tree::CallPath;
use crate::{Ident, TypeInfo};

#[derive(Debug, Clone)]
pub enum MethodName {
    /// Represents a method lookup with a type somewhere in the path
    FromType {
        call_path: CallPath<Ident>,
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
    FromTrait { call_path: CallPath<Ident> },
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
