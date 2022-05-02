use sway_types::Span;

use crate::parse_tree::CallPath;
use crate::{Ident, TypeInfo};

#[derive(Debug, Clone)]
pub enum MethodName {
    /// Represents a method lookup with a type somewhere in the path
    FromType {
        call_path: CallPath,
        type_name: Option<TypeInfo>,
        type_name_span: Option<Span>,
    },
    /// Represents a method lookup that does not contain any types in the path
    FromModule { method_name: Ident },
}

impl MethodName {
    /// To be used for error messages and debug strings
    pub fn easy_name(&self) -> Ident {
        match self {
            MethodName::FromType { call_path, .. } => call_path.suffix.clone(),
            MethodName::FromModule { method_name, .. } => method_name.clone(),
        }
    }
}
