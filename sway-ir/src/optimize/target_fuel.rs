//! Some Fuel VM specific utilities.
use crate::{
    context::Context,
    irtype::{Type, TypeContent},
};

/// Return whether a [Type] _cannot_ fit in a Fuel VM register and requires 'demotion'.
pub(super) fn is_demotable_type(context: &Context, ty: &Type) -> bool {
    match ty.get_content(context) {
        TypeContent::Unit
        | TypeContent::Bool
        | TypeContent::TypedPointer(_)
        | TypeContent::Pointer => false,
        TypeContent::Uint(bits) => *bits > 64,
        _ => true,
    }
}
