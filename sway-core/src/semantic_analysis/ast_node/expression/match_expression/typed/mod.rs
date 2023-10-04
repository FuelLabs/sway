//! This module type checks match expressions and desugars match expressions to if statements.
//!
//! Given the following simple example:
//!
//! ```ignore
//! struct Point {
//!     x: u64,
//!     y: u64
//! }
//!
//! let p = Point {
//!     x: 42,
//!     y: 24
//! };
//!
//! match p {
//!     Point { x, y: 5 } => { x },
//!     Point { x: 24, y } => { y },
//!     _ => 0
//! }
//! ```
//!
//! The resulting if statement would look roughly like this:
//!
//! ```ignore
//! let __NEW_NAME = p;
//! if __NEW_NAME.y == 5 {
//!     let x = __NEW_NAME.x;
//!     x
//! } else if __NEW_NAME.x == 24 {
//!     let y = __NEW_NAME.y;
//!     y
//! } else {
//!     0
//! }
//! ```
//! For more detailed examples see the TODO-IG.

mod matcher;
mod typed_match_branch;
mod typed_match_expression;
mod typed_scrutinee;

pub(crate) use matcher::ReqDeclTree;// TODO-IG: Remove. Replace with new Req struct.
