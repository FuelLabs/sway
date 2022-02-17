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
/*
mod error;
#[macro_use]
mod punctuated;
mod path;
mod item;
mod array;
mod ty;
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
/*
pub use error::*;
pub use punctuated::*;
pub use path::*;
pub use item::*;
pub use array::*;
pub use ty::*;
pub use generics::*;
pub use expr::*;
pub use pattern::*;
pub use assignable::*;
pub use dependency::*;
pub use program::*;
*/

//#[cfg(test)]
//mod test;
