mod abi;
pub mod auto_impl;
mod configurable;
mod constant;
#[allow(clippy::module_inception)]
mod declaration;
mod r#enum;
mod function;
mod impl_trait;
mod storage;
mod r#struct;
mod supertrait;
mod r#trait;
mod trait_fn;
mod trait_type;
mod type_alias;
mod variable;

pub(crate) use supertrait::*;
