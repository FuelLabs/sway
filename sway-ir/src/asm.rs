//! An 'asm' block represents an opaque set of Fuel VM assembly instructions, embedded in place and
//! intended to be inserted as is into the assembly code generation.
//!
//! An [`AsmBlock`] has symbols for arguments and an optional return name and contains a list of
//! [`AsmInstruction`].
//!
//! The syntax in Sway for asm blocks is shown by this example, and [`AsmBlock`] represents it
//! symbolically:
//!
//! ```text
//! asm(r1: self, r2: other, r3) {
//!     add r3 r2 r1;
//!     r3: u64
//! }
//! ```

use sway_types::ident::Ident;

use crate::{
    context::Context, irtype::Type, metadata::MetadataIndex, pretty::DebugWithContext, value::Value,
};

#[doc(hidden)]
#[derive(Clone, Debug, DebugWithContext)]
pub struct AsmBlock {
    pub args_names: Vec<Ident>,
    pub body: Vec<AsmInstruction>,
    pub return_type: Type,
    pub return_name: Option<Ident>,
}

#[derive(Clone, Debug)]
pub struct AsmArg {
    pub name: Ident,
    pub initializer: Option<Value>,
}

#[derive(Clone, Debug)]
pub struct AsmInstruction {
    pub op_name: Ident,
    pub args: Vec<Ident>,
    pub immediate: Option<Ident>,
    pub metadata: Option<MetadataIndex>,
}

impl AsmBlock {
    /// Create a new [`AsmBlock`] in the passed context and return its handle.
    pub fn new(
        args_names: Vec<Ident>,
        body: Vec<AsmInstruction>,
        return_type: Type,
        return_name: Option<Ident>,
    ) -> Self {
        AsmBlock {
            args_names,
            body,
            return_type,
            return_name,
        }
    }
}
