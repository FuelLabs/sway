mod abi;
mod configurable;
mod const_generic;
mod constant;
#[allow(clippy::module_inception)]
mod declaration;
mod r#enum;
mod function;
mod impl_trait;
mod storage;
mod r#struct;
mod r#trait;
mod trait_fn;
mod trait_type;
mod type_alias;
mod variable;

pub use abi::*;
pub use configurable::*;
pub use const_generic::*;
pub use constant::*;
pub use declaration::*;
pub use function::*;
pub use impl_trait::*;
pub use r#enum::*;
pub use r#struct::*;
pub use r#trait::*;
pub use storage::*;
pub use trait_fn::*;
pub use trait_type::*;
pub use type_alias::*;
pub use variable::*;

use crate::ast_elements::type_argument::GenericTypeArgument;

pub trait FunctionSignature {
    fn parameters(&self) -> &Vec<TyFunctionParameter>;
    fn return_type(&self) -> &GenericTypeArgument;
}
