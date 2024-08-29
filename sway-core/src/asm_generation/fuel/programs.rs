mod r#abstract;
mod allocated;
mod r#final;

pub(crate) use allocated::AllocatedProgram;
pub(crate) use r#abstract::{AbstractEntry, AbstractProgram};
pub(crate) use r#final::FinalProgram;

pub(crate) type SelectorOpt = Option<[u8; 4]>;
pub(crate) type FnName = String;
pub(crate) type ImmOffset = u64;
