#[allow(clippy::module_inception)]
mod declaration;
mod r#trait;

pub use declaration::*;
pub(crate) use r#trait::*;
