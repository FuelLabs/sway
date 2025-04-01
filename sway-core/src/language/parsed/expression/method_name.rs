use crate::engine_threading::{EqWithEngines, PartialEqWithEngines, PartialEqWithEnginesContext};
use crate::language::CallPath;
use crate::type_system::TypeBinding;
use crate::{GenericArgument, Ident, TypeId, TypeInfo};

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum MethodName {
    /// Represents a method lookup with a type somewhere in the path
    /// like `a::b::C::d()` with `C` being the type.
    FromType {
        call_path_binding: TypeBinding<CallPath<(TypeInfo, Ident)>>,
        method_name: Ident,
    },
    /// Represents a method lookup that does not contain any types in the path
    /// something like a.b(c)
    /// in this case, the first argument defines where to look for the method
    FromModule { method_name: Ident },
    /// something like a::b::c()
    /// in this case, the path defines where the fn symbol is defined
    /// used for things like std::ops::add(a, b).
    /// in this case, the first argument determines the type to look for
    FromTrait { call_path: CallPath },
    /// Represents a method lookup with a fully qualified path.
    /// like <S as Trait>::method()
    FromQualifiedPathRoot {
        ty: GenericArgument,
        as_trait: TypeId,
        method_name: Ident,
    },
}

impl EqWithEngines for MethodName {}
impl PartialEqWithEngines for MethodName {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        match (self, other) {
            (
                MethodName::FromType {
                    call_path_binding,
                    method_name,
                },
                MethodName::FromType {
                    call_path_binding: r_call_path_binding,
                    method_name: r_method_name,
                },
            ) => call_path_binding.eq(r_call_path_binding, ctx) && method_name == r_method_name,
            (
                MethodName::FromModule { method_name },
                MethodName::FromModule {
                    method_name: r_method_name,
                },
            ) => method_name == r_method_name,
            (
                MethodName::FromTrait { call_path },
                MethodName::FromTrait {
                    call_path: r_call_path,
                },
            ) => call_path == r_call_path,
            (
                MethodName::FromQualifiedPathRoot {
                    ty,
                    as_trait,
                    method_name,
                },
                MethodName::FromQualifiedPathRoot {
                    ty: r_ty,
                    as_trait: r_as_trait,
                    method_name: r_method_name,
                },
            ) => ty.eq(r_ty, ctx) && as_trait.eq(r_as_trait) && method_name == r_method_name,
            _ => false,
        }
    }
}

impl MethodName {
    /// To be used for error messages and debug strings
    pub fn easy_name(&self) -> Ident {
        match self {
            MethodName::FromType { method_name, .. } => method_name.clone(),
            MethodName::FromTrait { call_path, .. } => call_path.suffix.clone(),
            MethodName::FromModule { method_name, .. } => method_name.clone(),
            MethodName::FromQualifiedPathRoot { method_name, .. } => method_name.clone(),
        }
    }
}
