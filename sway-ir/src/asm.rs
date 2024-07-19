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

impl AsmInstruction {
    pub fn log_no_span(
        ra: impl Into<String>,
        rb: impl Into<String>,
        rc: impl Into<String>,
        rd: impl Into<String>,
    ) -> Self {
        AsmInstruction {
            op_name: Ident::new(sway_types::Span::from_string("log".into())),
            args: vec![
                Ident::new_no_span(ra.into()),
                Ident::new_no_span(rb.into()),
                Ident::new_no_span(rc.into()),
                Ident::new_no_span(rd.into()),
            ],
            immediate: None,
            metadata: None,
        }
    }

    pub fn lw_no_span(
        dst: impl Into<String>,
        src: impl Into<String>,
        offset: impl Into<String>,
    ) -> Self {
        AsmInstruction {
            op_name: Ident::new(sway_types::Span::from_string("lw".into())),
            args: vec![
                Ident::new_no_span(dst.into()),
                Ident::new_no_span(src.into()),
            ],
            immediate: Some(Ident::new_no_span(offset.into())),
            metadata: None,
        }
    }

    pub fn mul_no_span(dst: impl Into<String>, a: impl Into<String>, b: impl Into<String>) -> Self {
        AsmInstruction {
            op_name: Ident::new(sway_types::Span::from_string("mul".into())),
            args: vec![
                Ident::new_no_span(dst.into()),
                Ident::new_no_span(a.into()),
                Ident::new_no_span(b.into()),
            ],
            immediate: None,
            metadata: None,
        }
    }

    pub fn add_no_span(dst: impl Into<String>, a: impl Into<String>, b: impl Into<String>) -> Self {
        AsmInstruction {
            op_name: Ident::new(sway_types::Span::from_string("add".into())),
            args: vec![
                Ident::new_no_span(dst.into()),
                Ident::new_no_span(a.into()),
                Ident::new_no_span(b.into()),
            ],
            immediate: None,
            metadata: None,
        }
    }

    pub fn sub_no_span(dst: impl Into<String>, a: impl Into<String>, b: impl Into<String>) -> Self {
        AsmInstruction {
            op_name: Ident::new(sway_types::Span::from_string("sub".into())),
            args: vec![
                Ident::new_no_span(dst.into()),
                Ident::new_no_span(a.into()),
                Ident::new_no_span(b.into()),
            ],
            immediate: None,
            metadata: None,
        }
    }
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
