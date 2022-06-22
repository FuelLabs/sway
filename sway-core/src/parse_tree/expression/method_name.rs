use crate::parse_tree::CallPath;
use crate::type_engine::TypeBinding;
use crate::{Ident, TypeInfo};

#[derive(Debug, Clone)]
pub enum MethodName {
    /// Represents a method lookup with a type somewhere in the path
    /// a::b::~C::<T, E>::d::<F>(..)
    FromType {
        path_prefixes: Vec<Ident>,
        type_info_binding: TypeBinding<TypeInfo>,
        method_name: Ident,
    },
    /// Represents a method lookup that does not contain any types in the path
    /// something like a.b(c)
    /// in this case, the first argument defines where to look for the method
    FromModule { method_name: Ident },
    /// something like a::b::c()
    /// in this case, the path defines where the fn symbol is defined
    /// used for things like core::ops::add(a, b).
    /// in this case, the first argument determines the type to look for
    // TODO: turn this into a TypeBinding<CallPath> so that we can pass type arguments
    FromTrait { call_path: CallPath },
}

impl MethodName {
    /// To be used for error messages and debug strings
    pub fn easy_name(&self) -> Ident {
        match self {
            MethodName::FromType { method_name, .. } => method_name.clone(),
            MethodName::FromTrait { call_path, .. } => call_path.suffix.clone(),
            MethodName::FromModule { method_name, .. } => method_name.clone(),
        }
    }
}
