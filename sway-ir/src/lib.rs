//! Sway-IR is a library providing an intermediate representation for the
//! [Sway](https://github.com/FuelLabs/sway) compiler pipeline.
//!
//! It is inspired heavily by [LLVM](https://llvm.org/docs/LangRef.html) although it aims to remain
//! a much simpler system, providing only that which is required by Sway to target the Fuel virtual
//! machine.  It takes the form of a
//! [static single assignment](https://en.wikipedia.org/wiki/Static_single_assignment_form) graph
//! and is designed primarily to allow executable code optimization transforms powerful, yet remain
//! relatively simple.
//!
//! The core data type is [`Context`] which contains all the IR state in an entity-component style
//! system.  A [`Context`] contains one or more [`Module`]s, which in turn contain one or more
//! [`Function`]s. [`Function`]s have a set of arguments, contain a collection of local variables
//! and one or more [`Block`]s.  [`Block`]s contain lists of [`Instruction`]s and may be joined as a
//! graph to represent program control flow.
//!
//! Other important data types are [`Value`], [`Type`] and [`Constant`].  Function arguments, local
//! variables, instructions and constants are all [`Value`]s.
//!
//! The optimization passes are found in the [optimize] module.
//!
//! # Note:
//!
//! Most of the public data types used in this library are in fact wrappers around a handle into
//! the context.  The context uses the
//! [slotmap](https://github.com/orlp/slotmap) crate to maintain an entity
//! component system, or ECS.
//!
//! The nature of SSA is that it represents a graph of modules, functions, basic blocks and
//! instructions, which in Rust could be represented using references, [`Box`]es or [`std::rc::Rc`]
//! pointers.  But the nature of optimization passes are to transform these graphs into generally
//! smaller or at least somehow more efficient versions of themselves, and so to avoid a lot of
//! copying and the interior mutability problem Sway-IR uses the ECS.  Each handle implements
//! [`Copy`] and so is cheap to pass around by value, making changes to the ECS context simpler in
//! terms of satisfying the Rust borrow checker.

// For now it's easiest to just export absolutely everything to core_lang, we can refine the public
// API when it's closer to finished.

pub mod analysis;
pub use analysis::*;
pub mod asm;
pub use asm::*;
pub mod block;
pub use block::*;
pub mod constant;
pub use constant::*;
pub mod context;
pub use context::*;
pub mod error;
pub use error::*;
pub mod function;
pub use function::*;
pub mod instruction;
pub use instruction::*;
pub mod irtype;
pub use irtype::*;
pub mod metadata;
pub use metadata::*;
pub mod module;
pub use module::*;
pub mod optimize;
pub use optimize::*;
pub mod parser;
pub use parser::*;
pub mod variable;
pub use variable::*;
pub mod storage_key;
pub use storage_key::*;
pub mod pass_manager;
pub use pass_manager::*;
pub mod pretty;
pub use pretty::*;
pub mod printer;
pub use printer::*;
pub mod value;
pub use value::*;
pub mod verify;
pub use verify::*;
