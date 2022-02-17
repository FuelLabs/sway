#![recursion_limit = "256"]

pub enum Infallible {}

mod priv_prelude;
mod span;
mod parser;
mod combinators;
mod primitive;
mod ident;
mod tokens;
mod brackets;
mod literal;
mod punctuated;
mod ty;
/*
mod error;
#[macro_use]
mod path;
mod item;
mod array;
mod generics;
mod expr;
mod pattern;
mod assignable;
mod dependency;
mod program;
*/

pub use span::*;
pub use parser::*;
pub use combinators::*;
pub use primitive::*;
pub use ident::*;
pub use tokens::*;
pub use brackets::*;
pub use literal::*;
pub use punctuated::*;
pub use ty::*;
/*
pub use error::*;
pub use path::*;
pub use item::*;
pub use array::*;
pub use generics::*;
pub use expr::*;
pub use pattern::*;
pub use assignable::*;
pub use dependency::*;
pub use program::*;
*/

//#[cfg(test)]
//mod test;
