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
