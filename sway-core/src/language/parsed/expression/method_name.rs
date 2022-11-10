use sway_types::Span;

use crate::language::CallPath;
use crate::type_system::TypeBinding;
use crate::{Ident, TypeInfo};

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum MethodName {
    /// Represents a method lookup with a type somewhere in the path
    /// like `a::b::C::d()` with `C` being the type.
    FromType {
        call_path_binding: TypeBinding<CallPath<(TypeInfo, Span)>>,
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
