use crate::parse_tree::CallPath;
use crate::types::TypeInfo;
use crate::Ident;

#[derive(Debug, Clone)]
pub enum MethodName<'sc> {
    /// Represents a method lookup with a type somewhere in the path
    FromType {
        call_path: CallPath<'sc>,
        // if this is `None`, then use the first argument to determine the type
        type_name: Option<TypeInfo<'sc>>,
        is_absolute: bool,
    },
    /// Represents a method lookup that does not contain any types in the path
    FromModule { method_name: Ident<'sc> },
}

impl<'sc> MethodName<'sc> {
    /// To be used for error messages and debug strings
    pub(crate) fn easy_name(&self) -> &'sc str {
        match self {
            MethodName::FromType { call_path, .. } => call_path.suffix.primary_name,
            MethodName::FromModule { method_name, .. } => method_name.primary_name,
        }
    }
}
