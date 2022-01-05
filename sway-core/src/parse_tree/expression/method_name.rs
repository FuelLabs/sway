use crate::parse_tree::CallPath;
use crate::type_engine::TypeInfo;
use crate::Ident;

#[derive(Debug, Clone)]
pub enum MethodName {
    /// Represents a method lookup with a type somewhere in the path
    FromType {
        call_path: CallPath,
        // if this is `None`, then use the first argument to determine the type
        type_name: Option<TypeInfo>,
    },
    /// Represents a method lookup that does not contain any types in the path
    FromModule { method_name: Ident },
}

impl MethodName {
    /// To be used for error messages and debug strings
    pub(crate) fn easy_name(&self) -> Ident {
        match self {
            MethodName::FromType { call_path, .. } => call_path.suffix.clone(),
            MethodName::FromModule { method_name, .. } => method_name.clone(),
        }
    }
}
