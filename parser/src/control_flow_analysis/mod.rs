//!
//! This module contains all of the logic related to control flow analysis.
//!
//! # Synopsis of Algorithm
//!
//! The graph construction algorithm is as follows:
//!
//! ```ignore
//! For every node in the syntax tree:
//!   if it is non-branching:
//!      push it onto all current not-yet-terminated tree leaves, thus adding it to the end of every path
//!   else, if it is branching:
//!       fork all not-yet-terminated leaves to have two paths coming off of them
//!       in one path, put one of the node branches. in the other path, put the other node branch.
//!   else if it is a termination point (i.e. aborting of this path):
//!       mark the leaf node as terminated, preventing more nodes from being added.
//!
//! ```
//!
//! After the graph which models control flow is constructed, certain relationships are examined:
//! 1. exhaustive returns from functions
//! - TODO - ensure all terminating nodes from a function have the right type, and that no path
//! makes it to the end of the block without terminating
//! 1. dead code
//! - TODO -- boolean "reached" flag for every ast node
//!
//!
//! Using this dominator tree, it analyzes these qualities of the program:
//! 1. Node reachability
//! 1. Type correctness on all paths
//!
//! # # Terms
//! # # # Node
//! A node is any [crate::semantics::TypedAstNode], with some [crate::semantics::TypedAstNodeContent].
//! # # # Dominating nodes
//! A dominating node is a node which all previous nodes pass through. These are what we are
//! concerned about in control flow analysis. More formally,
//! A node _M_ dominates a node _N_ if every path from the entry that reaches node _N_ has to pass through node _M_.
//! # # # Reachability

mod dead_code_analysis;
mod flow_graph;
pub use dead_code_analysis::*;
pub use flow_graph::*;
