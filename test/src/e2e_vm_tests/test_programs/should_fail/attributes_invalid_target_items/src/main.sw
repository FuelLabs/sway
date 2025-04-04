#[storage(invalid)]
#[inline(invalid)]
#[test(invalid)]
/// Invalid outer comment.
#[payable(invalid)]
#[allow(invalid)]
#[cfg(invalid)]
#[deprecated(invalid)]
#[fallback(invalid)]
#[error_type(invalid)]
#[error(invalid)]
#[unknown(invalid)]
library;

// TODO: Extend with testing nested items once https://github.com/FuelLabs/sway/issues/6932 is implemented.

//! Invalid inner comment.

//! Invalid inner comment.
/// Invalid outer comment.

/// Invalid outer comment.
#[storage(invalid)]
#[inline(invalid)]
//! Invalid inner comment.
#[test(invalid)]
#[payable(invalid)]
/// Invalid outer comment.
//! Invalid inner comment.

//! Invalid inner comment.
/// Invalid outer comment.
#[allow(invalid)]
#[cfg(invalid)]
/// Invalid outer comment.
#[deprecated(invalid)]
#[fallback(invalid)]
#[error_type(invalid)]
#[error(invalid)]
//! Invalid inner comment.
/// Invalid outer comment.
mod module_kind;

mod type_alias_decl;

mod struct_decl;
mod struct_impl_assoc_const;
mod struct_impl_assoc_fn;
mod struct_impl_method;

mod enum_decl;
// TODO: Add test for associated constants once https://github.com/FuelLabs/sway/issues/6344 is implemented.
mod enum_impl_assoc_fn;
mod enum_impl_method;

mod trait_decl_assoc_const;
mod trait_decl_assoc_fn;
mod trait_decl_assoc_type;
mod trait_decl_method;
mod trait_decl_provided_assoc_fn;
mod trait_decl_provided_method;

mod trait_impl_for_enum;
mod trait_impl_for_struct;

mod ok_lib;
mod use_lib;

mod module_const;

mod module_fn;

mod comments;