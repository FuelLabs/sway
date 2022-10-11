//! This module type checks match expressions and desugars match expressions to if statements.
//!
//! Given the following example:
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
//!     Point { x, y: 24 } => { x },
//!     _ => 0
//! }
//! ```
//!
//! The resulting if statement would look roughly like this:
//!
//! ```ignore
//! let NEW_NAME = p;
//! if NEW_NAME.y==5 {
//!     let x = 42;
//!     x
//! } else if NEW_NAME.y==42 {
//!     let x = 42;
//!     x
//! } else {
//!     0
//! }
//! ```

mod matcher;
mod typed_match_branch;
mod typed_match_expression;
mod typed_scrutinee;

pub(crate) use matcher::MatchReqMap;
pub(crate) use typed_match_expression::TyMatchExpression;
